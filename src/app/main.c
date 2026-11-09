#include "app.h"
#include "board.h"

// Cold boot: ROM loads firmware into memory; start.S runs and calls here
int main(void) {
  board_init();
  app_init();
  app_run();
}

// Wake entry for board-configured deep sleep, reached through ROM and wake.S
// Assumes clock state is preserved in LP RAM; skips flash loading, state
// clearing, and full hardware initialization for much faster wake
void wake_main(void) {
  board_resume();
  app_run();
}
