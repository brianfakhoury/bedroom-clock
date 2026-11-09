#pragma once

void board_init(void);
void board_resume(void);

// Returns only if a pending touch cancels sleep
void board_sleep(void);
