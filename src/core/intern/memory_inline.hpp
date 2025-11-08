#pragma once

#include "core/assert.hpp"
#include "core/sys_types.hpp"

static inline bool mem_size_safe_multiply(size_t a, size_t b, size_t *result) {
  // A size_t with its high-half bits all set to 1
  const size_t high_bits = SIZE_MAX << (sizeof(size_t) * 8 / 2);
  *result = a * b;

  if (UNLIKELY(*result == 0)) {
    return (a == 0 || b == 0);
  }

  // To avoid having to do a divide, we'll look if both can be represented in N/2 bits
  return ((high_bits & (a | b)) == 0 || (*result / b == a));
}
