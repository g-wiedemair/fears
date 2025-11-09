#pragma once

#include "core/atomic_ops_utils.hpp"

#define NOGDI
#ifndef NOMINMAX
#  define NOMINMAX
#endif
#define WIN32_LEAN_AND_MEAN

#include <windows.h>

//-------------------------------------------------------------------------------------------------
// 32-bit operations

ATOMIC_INLINE uint32_t atomic_add_and_fetch_u32(uint32_t *p, uint32_t x) {
  return InterlockedExchangeAdd(p, x) + x;
}

//-------------------------------------------------------------------------------------------------
// 64-bit operations

ATOMIC_INLINE uint64_t atomic_add_and_fetch_u64(uint64_t *p, uint64_t x) {
  return InterlockedExchangeAdd64((int64_t *)p, (int64_t)x) + x;
}

ATOMIC_INLINE uint64_t atomic_cas_u64(uint64_t *v, uint64_t old, uint64_t _new) {
  return InterlockedCompareExchange64((int64_t *)v, _new, old);
}
