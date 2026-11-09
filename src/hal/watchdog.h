#pragma once
// Guard active work; interrupts_wait_until pauses the watchdog while waiting
void watchdog_arm(void);
void watchdog_disarm(void);
void watchdog_feed(void);
void watchdogs_disable(void);
