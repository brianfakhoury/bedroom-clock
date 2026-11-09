#pragma once

typedef struct {
  unsigned x, y, width, height;
} rect;

// Rotate a rectangle within the 480x480 screen by clockwise quarter turns
rect rect_rotate(rect area, unsigned rotation);
