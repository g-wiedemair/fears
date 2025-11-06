#include "memory_alloc_intern.hpp"

#include "core/assert.hpp"
#include "core/memory.hpp"
#include "core/sys_types.hpp"
#include <cassert>
#include <cstdarg>
#include <cstdio>

struct MemHead {
  // length of allocated memory block
  size_t len;
};
static_assert(MEM_MIN_CPP_ALIGNMENT <= alignof(MemHead), "Bad alignment of MemHead");
static_assert(MEM_MIN_CPP_ALIGNMENT <= sizeof(MemHead), "Bad size of MemHead");

struct MemHeadAligned {
  uint16_t alignment;
  size_t len;
};
static_assert(MEM_MIN_CPP_ALIGNMENT <= alignof(MemHeadAligned), "Bad alignment of MemHeadAligned");
static_assert(MEM_MIN_CPP_ALIGNMENT <= sizeof(MemHeadAligned), "Bad size of MemHeadAligned");

//-------------------------------------------------------------------------------------------------
// Locally used defines and vars

static bool alloc_debug_memset = false;
static void (*error_callback)(const char *) = nullptr;

/**
 * guarded alloc always allocate multiple of 4 bytes. That means that the lower 2 bits of the `len`
 * member of MemHead/MemHeadAligned can be used for the bitflags below
 */
enum {
  MEMHEAD_FLAG_ALIGN = 1 << 0,
  MEMHEAD_FLAG_FROM_CPP_NEW = 1 << 1,

  MEMHEAD_FLAG_MASK = (1 << 2) - 1,
};

#define MEMHEAD_FROM_PTR(ptr) (((MemHead *)ptr) - 1)
#define PTR_FROM_MEMHEAD(memh) (memh + 1)
#define MEMHEAD_ALIGNED_FROM_PTR(ptr) (((MemHeadAligned *)ptr) - 1)
#define MEMHEAD_IS_ALIGNED(memh) ((memh)->len & size_t(MEMHEAD_FLAG_ALIGN))
#define MEMHEAD_IS_FROM_CPP_NEW(memh) ((memh)->len & size_t(MEMHEAD_FLAG_FROM_CPP_NEW))
#define MEMHEAD_LEN(memh) ((memh)->len & ~size_t(MEMHEAD_FLAG_MASK))

#ifdef __GNUC__
__attribute__(format(printf, 1, 0)))
#endif
static void print_error(const char* message, va_list str_format_args) {
  char buf[512];
  vsnprintf(buf, sizeof(buf), message, str_format_args);
  buf[sizeof(buf) - 1] = '\0';

  if (error_callback) {
    error_callback(buf);
  } else {
    fputs(buf, stderr);
  }
}

#ifdef __GNUC__
__attribute__((format(printf, 1, 2)))
#endif
static void
print_error(const char *message, ...) {
  va_list str_format_args;
  va_start(str_format_args, message);
  print_error(message, str_format_args);
  va_end(str_format_args);
}

#ifdef __GNUC__
__attribute__((format(printf, 2, 3)))
#endif
static void
report_error_on_address(const void *mem, const char *message, ...) {
  va_list str_format_args;

  va_start(str_format_args, message);
  print_error(message, str_format_args);
  va_end(str_format_args);

  if (mem == nullptr) {
    mem_trigger_error_on_memory_block(nullptr, 0);
    return;
  }

  const MemHead *memh = MEMHEAD_FROM_PTR(mem);
  const usize len = MEMHEAD_LEN(memh);

  const void *address = memh;
  usize size = len + sizeof(*memh);
  if (UNLIKELY(MEMHEAD_IS_ALIGNED(memh))) {
    const MemHeadAligned *memh_aligned = MEMHEAD_ALIGNED_FROM_PTR(mem);
    address = MEMHEAD_REAL_PTR(memh_aligned);
    size = len + sizeof(*memh_aligned) + MEMHEAD_ALIGN_PADDING(memh_aligned->alignment);
  }
  mem_trigger_error_on_memory_block(address, size);
}

//-------------------------------------------------------------------------------------------------

void mem_lockfree_free(void *mem, internal::AllocationType allocation_type) {
  if (UNLIKELY(leak_detector_has_run)) {
    print_error("%s\n", free_after_leak_detection_message);
  }

  if (UNLIKELY(mem == nullptr)) {
    report_error_on_address(mem, "Attempt to free nullptr\n");
    return;
  }

  MemHead *memh = MEMHEAD_FROM_PTR(mem);
  size_t len = MEMHEAD_LEN(memh);

  if (allocation_type != internal::AllocationType::NEW_DELETE && MEMHEAD_IS_FROM_CPP_NEW(memh)) {
    report_error_on_address(
        mem, "Attempt to use C-style mem_free on a pointer created with CPP-style mem_new\n");
  }

  memory_usage_block_free(len);

  if (UNLIKELY(alloc_debug_memset && len)) {
    memset(memh + 1, 255, len);
  }
  if (UNLIKELY(MEMHEAD_IS_ALIGNED(memh))) {
    MemHeadAligned *memh_aligned = MEMHEAD_ALIGNED_FROM_PTR(mem);
    aligned_free(MEMHEAD_REAL_PTR(memh_aligned));
  } else {
    free(memh);
  }
}

void *mem_lockfree_malloc_aligned(size_t len,
                                  size_t alignment,
                                  const char *str,
                                  internal::AllocationType allocation_type) {
  // Huge alignment values doesn't make sense and wouldn't fit into uint16_t
  assert(alignment < 1024);

  // We only support alignments that are a power of two
  assert(IS_POW2(alignment));

  // Some OS specific allocators require a certain minimal alignment
  if (alignment < ALIGNED_ALLOC_MINIMUM_ALIGNMENT) {
    alignment = ALIGNED_ALLOC_MINIMUM_ALIGNMENT;
  }

  // It's possible that MemHead's size is not properly aligned
  size_t extra_padding = MEMHEAD_ALIGN_PADDING(alignment);

#ifdef WITH_MEM_VALGRIND
  TODO;
#endif

  len = SIZET_ALIGN_4(len);

  MemHeadAligned *memh = (MemHeadAligned *)aligned_alloc(
      len + extra_padding + sizeof(MemHeadAligned), alignment);

  if (LIKELY(memh)) {
    memh = (MemHeadAligned *)((char *)memh + extra_padding);

    if (LIKELY(len)) {
      if (UNLIKELY(alloc_debug_memset)) {
        memset(memh + 1, 255, len);
      }
#ifdef WITH_MEM_VALGRIND
      TODO;
#endif
    }

    memh->len = len | size_t(MEMHEAD_FLAG_ALIGN) |
                size_t(allocation_type == internal::AllocationType::NEW_DELETE ?
                           MEMHEAD_FLAG_FROM_CPP_NEW :
                           0);

    memh->alignment = uint16_t(alignment);
    memory_usage_block_alloc(len);

    return PTR_FROM_MEMHEAD(memh);
  }

  print_error("Alloc returns null: len=" SIZET_FORMAT " in %s, total " SIZET_FORMAT "\n",
              SIZET_ARG(len),
              str,
              memory_usage_current());
  return nullptr;
}

void *mem_lockfree_calloc(size_t len, const char *str) {
  MemHead *memh;
  len = SIZET_ALIGN_4(len);
  memh = (MemHead *)calloc(1, len + sizeof(MemHead));

  if (LIKELY(memh)) {
    memh->len = len;
    memory_usage_block_alloc(len);
    return PTR_FROM_MEMHEAD(memh);
  }

  print_error("Calloc returns null: len=" SIZET_FORMAT " in %s, total " SIZET_FORMAT "\n",
              SIZET_ARG(len),
              str,
              memory_usage_current());
  return nullptr;
}

void mem_lockfree_print_memlist() {}

void mem_lockfree_set_error_callback(void (*func)(const char *)) {
  error_callback = func;
}

usize mem_lockfree_get_memory_in_use() {
  return memory_usage_current();
}

uint32_t mem_lockfree_get_memory_blocks_in_use() {
  return uint32_t(memory_usage_block_num());
}

void mem_lockfree_clear_memlist() {}
