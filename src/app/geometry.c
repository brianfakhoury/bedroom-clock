#include "geometry.h"

rect rect_rotate(rect area, unsigned rotation) {
  unsigned x = area.x, y = area.y, w = area.width, h = area.height;
  switch (rotation & 3) {
    case 1:
      return (rect){480 - y - h, x, h, w};
    case 2:
      return (rect){480 - x - w, 480 - y - h, w, h};
    case 3:
      return (rect){y, 480 - x - w, h, w};
    default:
      return area;
  }
}
