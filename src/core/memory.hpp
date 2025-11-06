#pragma once

#include "core/core_api.hpp"

#include <new>

#include "core/intern/memory_function_pointers.hpp"
#include <type_traits>

/**
 * Conservative value of memory alignment returned by non-aligned OS-level
 * memory allocation functions.
 */
#define MEM_MIN_CPP_ALIGNMENT \
  (__STDCPP_DEFAULT_NEW_ALIGNMENT__ < alignof(void *) ? __STDCPP_DEFAULT_NEW_ALIGNMENT__ : \
                                                        alignof(void *))

/**
 * Release memory previously allocated by the C-style functions of this module
 * It is illegal to call this function with data allocated by #mem_new
 */
CORE_API void mem_free(void *vmemh);

/**
 * Allocate a block of memory of size len, with tag name str. The
 * memory is cleared. The name must be static, because only a
 * pointer to it is stored!
 */
CORE_API void *mem_calloc(size_t len, const char *str);

//-------------------------------------------------------------------------------------------------

// Print a list of the names and sizes of all allocated memory blocks.
extern void (*mem_print_memlist)();
// Set the callback function for error output
extern void (*mem_set_error_callback)(void (*func)(const char *));
// Memory usage stats
extern size_t (*mem_get_memory_in_use)();
// get amount of memory blocks in use
extern uint32_t (*mem_get_memory_blocks_in_use)();

/**
 * This should be called as early as possible in the program. When it has been called, information
 * about memory leaks will be printed on exit.
 */
CORE_API void mem_init_memleak_detection();

/**
 * Switch allocator to slow fully guarded mode.
 *
 * Use for debug purposes. This allocator contains lock section around every allocator call, which
 * makes it slow. What is gained with this is the ability to have list of allocated blocks (in an
 * addition to the tracking of number of allocations and amount of allocated bytes).
 *
 * \note The switch between allocator types can only happen before any allocation did happen.
 */
CORE_API void mem_use_guarded_allocator();

//-------------------------------------------------------------------------------------------------

/**
 * Allocate new memory for an object of type T, and construct it.
 * delete must be used to mem_delete the object. Calling free on it is illegal.
 *
 * Do not assume that this ever zero-initializes memory
 */
template<typename T, typename... Args>
inline T *mem_new(const char *allocation_name, Args &&...args) {
  void *buffer = internal::mem_malloc_aligned(
      sizeof(T), alignof(T), allocation_name, internal::AllocationType::NEW_DELETE);
  return new (buffer) T(std::forward<Args>(args)...);
}

/**
 * Destruct and deallocate an object previously allocated and constructed with mem_new.
 */
template<typename T> inline void mem_delete(const T *ptr) {
  static_assert(
      !std::is_void_v<T>,
      "mem_delete on a void pointer is not possible, 'static_cast' it to the correct type");
  if (ptr == nullptr) {
    return;
  }

  ptr->~T();
  internal::mem_free(const_cast<T *>(ptr), internal::AllocationType::NEW_DELETE);
}

//-------------------------------------------------------------------------------------------------

/**
 * Define overloaded new/delete operators for C++ types
 */
