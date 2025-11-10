#pragma once

#include "core/assert.hpp"
#include <cstdint>

#define STRINGIFY_APPEND(a, b) "" a #b
#define STRINGIFY(x) STRINGIFY_APPEND("", x)

#define STREQ(a, b) (strcmp(a, b) == 0)

#define AT __FILE__ ":" STRINGIFY(__LINE__)

#define ENABLE_IF(condition) typename std::enable_if_t<(condition)> * = nullptr

#if defined(_MSC_VER) && !defined(__clang__)
#  define FE_NO_UNIQUE_ADDRESS [[msvc::no_unique_address]]
#elif defined(__has_cpp_attribute)
#  if __has_cpp_attribute(no_unique_address)
#    define FE_NO_UNIQUE_ADDRESS [[no_unique_address]]
#  else
#    define FE_NO_UNIQUE_ADDRESS
#  endif
#else
#  define FE_NO_UNIQUE_ADDRESS [[no_unique_address]]
#endif

inline constexpr int64_t is_power_of_2(const int64_t x) {
  fassert(x >= 0);
  return (x & (x - 1)) == 0;
}

inline constexpr int64_t log2_floor(const int64_t x) {
  fassert(x >= 0);
  return x <= 1 ? 0 : 1 + log2_floor(x >> 1);
}

inline constexpr int64_t log2_ceil(const int64_t x) {
  fassert(x >= 0);
  return (is_power_of_2(int64_t(x))) ? log2_floor(x) : log2_floor(x) + 1;
}

inline constexpr int64_t power_of_2_max(const int64_t x) {
  fassert(x >= 0);
  return 1ll << log2_ceil(x);
}
