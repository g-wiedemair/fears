#pragma once

#include <atomic>

#if defined(_MSC_VER)
#  define NOGDI
#  ifndef NOMINMAX
#    define NOMINMAX
#  endif
#  define WIN32_LEAN_AND_MEAN
#  include <intrin.h>
#  include <windows.h>
#endif

#if defined(_MSC_VER)
#  define ATOMIC_INLINE static __forceinline
#else
#  define ATOMIC_INLINE static inline __attribute__((always_inline))
#endif

#if (UINT_MAX == 0xFFFFFFFF)
#  define LG_SIZEOF_INT 4
#elif (UINT_MAX == 0xFFFFFFFFFFFFFFFF)
#  define LG_SIZEOF_INT 8
#else
#  error "Cannot find int size"
#endif

#if defined(__SIZEOF_POINTER__)
#  define LG_SIZEOF_PTR __SIZEOF_POINTER__
#elif defined(UINTPTR_MAX)
#  if (UINTPTR_MAX == 0xFFFFFFFF)
#    define LG_SIZEOF_PTR 4
#  elif (UINTPTR_MAX == 0xFFFFFFFFFFFFFFFF)
#    define LG_SIZEOF_PTR 8
#  endif
#elif defined(__WORDSIZE) /* Fallback for older glibc and cpp */
#  if (__WORDSIZE == 32)
#    define LG_SIZEOF_PTR 4
#  elif (__WORDSIZE == 64)
#    define LG_SIZEOF_PTR 8
#  endif
#endif

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

//-------------------------------------------------------------------------------------------------

ATOMIC_INLINE uint32_t atomic_add_and_fetch_u(uint32_t *p, uint32_t x) {
#if (LG_SIZEOF_INT == 8)
  return (uint32_t)atomic_add_and_fetch_u64((uint64_t *)p, (uint64_t)x);
#elif (LG_SIZEOF_INT == 4)
  return (uint32_t)atomic_add_and_fetch_u32((uint32_t *)p, (uint32_t)x);
#endif
}

ATOMIC_INLINE uint32_t atomic_sub_and_fetch_u(uint32_t *p, uint32_t x) {
#if (LG_SIZEOF_INT == 8)
  return (uint32_t)atomic_add_and_fetch_u64((uint64_t *)p, (uint64_t)-((int64_t)x));
#elif (LG_SIZEOF_INT == 4)
  return (uint32_t)atomic_add_and_fetch_u32((uint32_t *)p, (uint32_t)-((int32_t)x));
#endif
}

ATOMIC_INLINE size_t atomic_add_and_fetch_z(size_t *p, size_t x) {
#if (LG_SIZEOF_PTR == 8)
  return (size_t)atomic_add_and_fetch_u64((uint64_t *)p, (uint64_t)x);
#elif (LG_SIZEOF_PTR == 4)
  return (size_t)atomic_fetch_and_add_u32((uint32_t *)p, (uint32_t)x);
#endif
}

ATOMIC_INLINE size_t atomic_sub_and_fetch_z(size_t *p, size_t x) {
#if (LG_SIZEOF_PTR == 8)
  return (size_t)atomic_add_and_fetch_u64((uint64_t *)p, (uint64_t)-((int64_t)x));
#elif (LG_SIZEOF_PTR == 4)
  return (size_t)atomic_add_and_fetch_u32((uint32_t *)p, (uint32_t)-((int32_t)x));
#endif
}
