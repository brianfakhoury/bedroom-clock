#include <stddef.h>

// Keep memcpy/memset available through LTO for compiler-generated calls
__attribute__((used)) void* memcpy(void* dst, const void* src, size_t size) {
  unsigned char* d = dst;
  const unsigned char* s = src;
  while (size--) *d++ = *s++;
  return dst;
}

__attribute__((used)) void* memset(void* dst, int value, size_t size) {
  unsigned char* d = dst;
  while (size--) *d++ = (unsigned char)value;
  return dst;
}
