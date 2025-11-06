#include "string.hpp"

#include "core/assert.hpp"
#include <cstdio>

size_t fsnprintf(char *__restrict dst, size_t dst_maxncpy, const char *__restrict format, ...) {
  string_debug_size(dst, dst_maxncpy);

  va_list arg;
  va_start(arg, format);
  const size_t n = fvsnprintf(dst, dst_maxncpy, format, arg);
  va_end(arg);

  return n;
}

size_t fvsnprintf(char *dst, size_t dst_maxncpy, const char *format, va_list arg) {
  string_debug_size(dst, dst_maxncpy);

  fassert(dst != nullptr);
  fassert(dst_maxncpy > 0);
  fassert(format != nullptr);

  const size_t n = size_t(vsnprintf(dst, dst_maxncpy, format, arg));
  if (n < dst_maxncpy) {
    dst[n] = '\0';
  } else {
    dst[dst_maxncpy - 1] = '\0';
  }

  return n;
}
