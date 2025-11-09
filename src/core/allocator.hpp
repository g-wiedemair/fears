#pragma once

#include "core/memory.hpp"

class GuardedAllocator {
 public:
  void *allocate(size_t size, size_t alignment, const char *name) {
    return mem_malloc_aligned(size, alignment, name);
  }

  void deallocate(void *ptr) {
    mem_free(ptr);
  }
};
