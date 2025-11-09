#pragma once

#include <atomic>

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
