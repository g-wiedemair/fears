#pragma once

#include <any>

namespace internal {

enum class AllocationType {
  /* Allocation is handled through 'C type' alloc/free calls */
  ALLOC_FREE,
  /* Allocation is handled through 'C++ type' new/delete calls */
  NEW_DELETE,
};

/** Internal implementation of mem_free */
extern void (*mem_free)(void *mem, AllocationType allocation_type);

/** Internal implementation of mem_malloc_aligned */
extern void *(*mem_malloc_aligned)(size_t len,
                                   size_t alignment,
                                   const char *str,
                                   AllocationType allocation_type);

/** Internal implementation of mem_calloc */
extern void *(*mem_calloc)(size_t len, const char *str);

/**
 * Store a std::any into a static opaque storage vector. The only purpose of this call is to
 * control the lifetime of the given data, there is no way to access it from here afterwards. User
 * code is expected to keep its own reference to the data contained in the `std::any` as long as it
 * needs it.
 *
 * Typically, this `any` should contain a `shared_ptr` to the actual data, to ensure that the data
 * itself is not duplicated, and that the static storage does become an owner of it.
 *
 * That way, the memleak data does not get destructed before the static storage is. Since this
 * storage is created before the memleak detection data (see the implementation of
 * #MEM_init_memleak_detection), it is guaranteed to happen after the execution and destruction of
 * the memleak detector.
 */
void add_memleak_data(std::any data);

}  // namespace internal
