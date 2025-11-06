#include "memory_function_pointers.hpp"

#include "core/memory.hpp"
#include "memory_alloc_intern.hpp"

#include <cassert>

void (*internal::mem_free)(void *mem, internal::AllocationType allocationType) = mem_lockfree_free;
void *(*internal::mem_malloc_aligned)(size_t len,
                                      size_t alignment,
                                      const char *str,
                                      internal::AllocationType allocation_type) =
    mem_lockfree_malloc_aligned;
void *(*internal::mem_calloc)(size_t len, const char *str) = mem_lockfree_calloc;

void (*mem_print_memlist)() = mem_lockfree_print_memlist;
void (*mem_set_error_callback)(void (*func)(const char *)) = mem_lockfree_set_error_callback;
size_t (*mem_get_memory_in_use)() = mem_lockfree_get_memory_in_use;
uint32_t (*mem_get_memory_blocks_in_use)() = mem_lockfree_get_memory_blocks_in_use;

void (*mem_clear_memlist)() = mem_lockfree_clear_memlist;

//-------------------------------------------------------------------------------------------------

void *aligned_alloc(size_t size, size_t alignment) {
  assert(alignment >= ALIGNED_ALLOC_MINIMUM_ALIGNMENT);

#ifdef WIN32
  return _aligned_malloc(size, alignment);

#else
  TODO;
#endif
}

void aligned_free(void *ptr) {
#ifdef WIN32
  _aligned_free(ptr);
#else
  free(ptr);
#endif
}

void mem_free(void *memh) {
  return internal::mem_free(memh, internal::AllocationType::ALLOC_FREE);
}

void *mem_calloc(size_t len, const char *str) {
  return internal::mem_calloc(len, str);
}

static void assert_for_allocator_change() {
  assert(mem_get_memory_blocks_in_use() == 0);
}

void mem_use_guarded_allocator() {
  assert_for_allocator_change();

  internal::mem_free = mem_guarded_free;
  internal::mem_malloc_aligned = mem_guarded_malloc_aligned;
  internal::mem_calloc = mem_guarded_calloc;

  mem_print_memlist = mem_guarded_print_memlist;
  mem_set_error_callback = mem_guarded_set_error_callback;
  mem_get_memory_in_use = mem_guarded_get_memory_in_use;
  mem_get_memory_blocks_in_use = mem_guarded_get_memory_blocks_in_use;
  mem_clear_memlist = mem_guarded_clear_memlist;
}
