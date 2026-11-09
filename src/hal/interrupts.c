#include "interrupts.h"

#include "registers.h"
#include "timer.h"
#include "watchdog.h"

static uint32_t event_pins;

void interrupts_init(uint32_t gpio_mask) {
  event_pins = gpio_mask;
  // WFI pauses the CPU clock without entering light sleep
  // Global interrupts stay masked so events are handled here, not in an ISR
  REG(PCR_CPU_WAITI) &= ~BIT(3);
  REG(PLIC_ENABLE) = 0;
  REG(PLIC_TYPE) = 0;
  REG(PLIC_PRIORITY1) = 1;
  REG(PLIC_THRESHOLD) = 0;
  REG(IRQ_MAP_GPIO) = 1;
  REG(IRQ_MAP_USB) = 1;
  REG(IRQ_MAP_SYSTIMER0) = 1;
  REG(USB_IRQ_ENABLE) = BIT(2);  // USB received packet only
  for (unsigned pin = 0; pin < 32; pin++)
    if (event_pins & BIT(pin)) REG(GPIO_PIN0 + pin * 4) = BIT(13) | (2u << 7);
  REG(GPIO_STATUS_CLEAR) = event_pins;
  REG(SYSTIMER_IRQ_ENABLE) = 1;
  REG(PLIC_ENABLE) = BIT(1);
  __asm__ volatile("csrw mie, %0" ::"r"(0xffffffffu));
}

interrupt_events interrupts_wait_until(uint64_t deadline) {
  REG(SYSTIMER_CONFIG) &= ~BIT(24);
  REG(SYSTIMER_IRQ_CLEAR) = 1;
  if (deadline) {
    if (ticks() >= deadline) return (interrupt_events){.timer = true};
    REG(SYSTIMER_TARGET0_CONFIG) = 0;
    REG(SYSTIMER_TARGET0_HIGH) = deadline >> 32;
    REG(SYSTIMER_TARGET0_LOW) = (uint32_t)deadline;
    REG(SYSTIMER_TARGET0_LOAD) = 1;
    REG(SYSTIMER_CONFIG) |= BIT(24);
  }
  // Check unread USB bytes explicitly; other events remain latched for WFI
  bool ready = (REG(USB_FIFO_CONFIG) & 4) || (deadline && ticks() >= deadline);
  if (!ready) {
    watchdog_disarm();
    __asm__ volatile("fence iorw, iorw; wfi" ::: "memory");
    watchdog_arm();
  }
  interrupt_events events = {
      .gpio = REG(GPIO_STATUS) & event_pins,
      .usb = (REG(USB_FIFO_CONFIG) & 4) != 0,
      .timer = deadline && ticks() >= deadline,
  };
  REG(GPIO_STATUS_CLEAR) = events.gpio;
  if (REG(USB_IRQ_RAW) & BIT(2)) {
    REG(USB_IRQ_CLEAR) = BIT(2);
    events.usb = true;
  }
  if (REG(SYSTIMER_IRQ_RAW) & 1) {
    REG(SYSTIMER_IRQ_CLEAR) = 1;
    events.timer = true;
  }
  REG(SYSTIMER_CONFIG) &= ~BIT(24);
  return events;
}
