#pragma once
// 0..100 is a gauge estimate; any larger value hides an unavailable reading
void battery_indicator_draw(unsigned percentage, unsigned rotation);
