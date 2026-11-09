#pragma once

// Startup-only code and data live in HP RAM, which is lost during deep sleep
#ifdef __riscv
#define BOOT_ONLY __attribute__((section(".init.code"), noinline))
#define BOOT_DATA __attribute__((section(".init.data")))
#else
#define BOOT_ONLY
#define BOOT_DATA
#endif
