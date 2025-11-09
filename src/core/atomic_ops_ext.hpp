#pragma once

#include "core/atomic_ops_utils.hpp"

#if defined(_MSC_VER)
#  include "core/intern/atomic_ops_msvc.hpp"
#endif

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

//-------------------------------------------------------------------------------------------------
// Pointer operations

ATOMIC_INLINE void *atomic_cas_ptr(void **v, void *old, void *_new) {
#if (LG_SIZEOF_PTR == 8)
  return (void *)atomic_cas_u64((uint64_t *)v, *(uint64_t *)&old, *(uint64_t *)&_new);
#elif (LG_SIZEOF_PTR == 4)
  return (void *)atomic_cas_u32((uint32_t *)v, *(uint32_t *)&old, *(uint32_t *)&_new);
#endif
}
