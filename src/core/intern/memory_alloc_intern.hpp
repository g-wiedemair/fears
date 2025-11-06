#pragma once

#include "memory_function_pointers.hpp"

#define ALIGNED_ALLOC_MINIMUM_ALIGNMENT sizeof(void *)

/* Exgtra padding which needs to be applied on MemHead to make it aligned */
#define MEMHEAD_ALIGN_PADDING(alignment) \
  ((size_t)alignment - (sizeof(MemHeadAligned) % (size_t)alignment))

/* Real pointer returned by the `malloc` or `aligned_alloc`. */
#define MEMHEAD_REAL_PTR(memh) ((char *)memh - MEMHEAD_ALIGN_PADDING(memh->alignment))

#define SIZET_ALIGN_4(len) ((len + 3) & ~(size_t)3)
#define IS_POW2(a) (((a) & ((a) - 1)) == 0)

#define SIZET_FORMAT "%zu"
#define SIZET_ARG(a) ((usize)(a))

#ifdef _MSC_VER
#  define MEM_INLINE static __inline
#else
#  define MEM_INLINE static inline
#endif

extern bool leak_detector_has_run;
extern char free_after_leak_detection_message[];

void *aligned_alloc(size_t size, size_t alignment);
void aligned_free(void *ptr);

void memory_usage_init();
void memory_usage_block_alloc(size_t size);
void memory_usage_block_free(size_t size);
size_t memory_usage_block_num();
size_t memory_usage_current();

/*
 * Clear the listbase of allocated memory blocks
 *
 * WARNING: This will make the whole guardedalloc system fully inconsistent. It is only indented to
 * be called in one place: the destructor of the #MemLeakPrinter class, which is only
 * instantiated once as a static variable by #mem_init_memleak_detection, and therefore destructed
 * once at program exit.
 */
extern void (*mem_clear_memlist)();

//-------------------------------------------------------------------------------------------------

/**
 * Prototypes for counted allocator functions
 */

void mem_lockfree_free(void *mem, internal::AllocationType allocation_type);
void *mem_lockfree_malloc_aligned(size_t len,
                                  size_t alignment,
                                  const char *str,
                                  internal::AllocationType allocation_type);
void *mem_lockfree_calloc(size_t len, const char *str);

void mem_lockfree_print_memlist();
void mem_lockfree_set_error_callback(void (*func)(const char *));
size_t mem_lockfree_get_memory_in_use();
uint32_t mem_lockfree_get_memory_blocks_in_use();

void mem_lockfree_clear_memlist();

//-------------------------------------------------------------------------------------------------

/**
 * Prototypes for guarded allocator functions
 */

void mem_guarded_free(void *mem, internal::AllocationType allocation_type);
void *mem_guarded_malloc_aligned(size_t len,
                                 size_t alignment,
                                 const char *str,
                                 internal::AllocationType allocation_type);
void *mem_guarded_calloc(size_t len, const char *str);

void mem_guarded_print_memlist();
void mem_guarded_set_error_callback(void (*func)(const char *));
size_t mem_guarded_get_memory_in_use();
uint32_t mem_guarded_get_memory_blocks_in_use();

void mem_guarded_clear_memlist();

//-------------------------------------------------------------------------------------------------

/**
 * Util to trigger an error for the given memory block
 */
#ifdef WITH_ASAN
TODO;

#else
MEM_INLINE void mem_trigger_error_on_memory_block(const void *, const size_t) {
#  ifdef WITH_ASSERT_ABORT
  abort();
#  endif
}
#endif
