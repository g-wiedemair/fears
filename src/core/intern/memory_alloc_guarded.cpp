#include "memory_alloc_intern.hpp"

#include "core/assert.hpp"
#include "core/atomic_ops_ext.hpp"
#include "core/intern/memory_inline.hpp"
#include "core/memory.hpp"
#include "core/sys_types.hpp"

#include <cassert>
#include <cstdarg>
#include <pthread.h>

struct LocalLink {
  LocalLink *next, *prev;
};

struct LocalListBase {
  void *first, *last;
};

namespace {

struct MemHead {
  int tag1;
  size_t len;
  MemHead *next, *prev;
  const char *name;
  const char *next_name;
  int tag2;
  uint16_t flag;
  uint16_t alignment;
};
static_assert(MEM_MIN_CPP_ALIGNMENT <= alignof(MemHead), "Bad alignment of MemHead");
static_assert(MEM_MIN_CPP_ALIGNMENT <= sizeof(MemHead), "Bad size of MemHead");

typedef MemHead MemHeadAligned;

}  // namespace

enum MemHeadFlag {
  MEMHEAD_FLAG_FROM_CPP_NEW = 1 << 1,
};

struct MemTail {
  int tag3, pad;
};

//-------------------------------------------------------------------------------------------------
// Locally used defines and vars

/* NOTE: this is endianness-sensitive. */
#define MAKE_ID(a, b, c, d) (int(d) << 24 | int(c) << 16 | (b) << 8 | (a))

#define MEMTAG1 MAKE_ID('M', 'E', 'M', 'O')
#define MEMTAG2 MAKE_ID('R', 'Y', 'B', 'L')
#define MEMTAG3 MAKE_ID('O', 'C', 'K', '!')
#define MEMFREE MAKE_ID('F', 'R', 'E', 'E')

#define MEMNEXT(x) ((MemHead *)(((char *)x) - offsetof(MemHead, next)))

static uint32_t tot_block = 0;
static size_t mem_in_use = 0, peak_mem = 0;

static volatile LocalListBase _mem_base;
static volatile LocalListBase *mem_base = &_mem_base;

static bool malloc_debug_memset = false;
static void (*error_callback)(const char *) = nullptr;

#ifdef __GNUC__
__attribute__((format(printf, 1, 0)))
#endif
static void
print_error(const char *message, va_list str_format_args) {
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

static void print_memory_error(const char *block, const char *error) {
  print_error("MemoryBlock %s: %s\n", block, error);

#ifdef WITH_ASSERT_ABORT
  abort();
#endif
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

  const MemHead *memh = static_cast<const MemHead *>(mem);
  memh--;
  const size_t len = memh->len;

  const void *address = memh;
  size_t size = len + sizeof(*memh) + sizeof(MemTail);
  if (UNLIKELY(memh->alignment > 0)) {
    const MemHeadAligned *memh_aligned = memh;
    address = MEMHEAD_REAL_PTR(memh_aligned);
    size = len + sizeof(*memh_aligned) + MEMHEAD_ALIGN_PADDING(memh_aligned->alignment) +
           sizeof(MemTail);
  }
  mem_trigger_error_on_memory_block(address, size);
}

static pthread_mutex_t thread_lock = PTHREAD_MUTEX_INITIALIZER;

static void mem_lock_thread() {
  pthread_mutex_lock(&thread_lock);
}
static void mem_unlock_thread() {
  pthread_mutex_unlock(&thread_lock);
}

static void add_tail(volatile LocalListBase *list_base, void *vlink) {
  LocalLink *link = static_cast<LocalLink *>(vlink);

  link->next = nullptr;
  link->prev = static_cast<LocalLink *>(list_base->last);

  if (list_base->last) {
    ((LocalLink *)list_base->last)->next = link;
  }
  if (list_base->first == nullptr) {
    list_base->first = link;
  }
  list_base->last = link;
}

static void make_memhead_header(MemHead *memh,
                                size_t len,
                                const char *str,
                                const internal::AllocationType allocation_type) {
  MemTail *memt;

  memh->tag1 = MEMTAG1;
  memh->name = str;
  memh->next_name = nullptr;
  memh->len = len;
  memh->flag = (allocation_type == internal::AllocationType::NEW_DELETE) ?
                   MEMHEAD_FLAG_FROM_CPP_NEW :
                   0;
  memh->alignment = 0;
  memh->tag2 = MEMTAG2;

#ifdef DEBUG_MEMDUPLINAME
  TODO;
#endif

#ifdef DEBUG_BACKTRACE_EXECINFO
  TODO;
#endif

  memt = (MemTail *)(((char *)memh) + sizeof(MemHead) + len);
  memt->tag3 = MEMTAG3;

  atomic_add_and_fetch_u(&tot_block, 1);
  atomic_add_and_fetch_z(&mem_in_use, len);

  mem_lock_thread();
  add_tail(mem_base, &memh->next);
  if (memh->next) {
    memh->next_name = MEMNEXT(memh->next)->name;
  }
  peak_mem = mem_in_use > peak_mem ? mem_in_use : peak_mem;
  mem_unlock_thread();
}

static const char *check_memlist(const MemHead *memh) {
  MemHead *forw, *back, *forwok, *backok;
  const char *name;

  forw = static_cast<MemHead *>(mem_base->first);
  if (forw) {
    forw = MEMNEXT(forw);
  }
  forwok = nullptr;
  while (forw) {
    if (forw->tag1 != MEMTAG1 || forw->tag2 != MEMTAG2) {
      break;
    }
    forwok = forw;
    if (forw->next) {
      forw = MEMNEXT(forw->next);
    } else {
      forw = nullptr;
    }
  }

  back = (MemHead *)mem_base->last;
  if (back) {
    back = MEMNEXT(back);
  }
  backok = nullptr;
  while (back) {
    if (back->tag1 != MEMTAG1 || back->tag2 != MEMTAG2) {
      break;
    }
    backok = back;
    if (back->prev) {
      back = MEMNEXT(back->prev);
    } else {
      back = nullptr;
    }
  }

  if (forw != back) {
    return ("MORE THAN 1 MEMORYBLOCK CORRUPT");
  }

  if (forw == nullptr && back == nullptr) {
    /* No wrong headers found then but in search of memblock */
    forw = static_cast<MemHead *>(mem_base->first);
    if (forw) {
      forw = MEMNEXT(forw);
    }
    forwok = nullptr;
    while (forw) {
      if (forw == memh) {
        break;
      }
      if (forw->tag1 != MEMTAG1 || forw->tag2 != MEMTAG2) {
        break;
      }
      forwok = forw;
      if (forw->next) {
        forw = MEMNEXT(forw->next);
      } else {
        forw = nullptr;
      }
    }
    if (forw == nullptr) {
      return nullptr;
    }

    back = (MemHead *)mem_base->last;
    if (back) {
      back = MEMNEXT(back);
    }
    backok = nullptr;
    while (back) {
      if (back == memh) {
        break;
      }
      if (back->tag1 != MEMTAG1 || back->tag2 != MEMTAG2) {
        break;
      }
      backok = back;
      if (back->prev) {
        back = MEMNEXT(back->prev);
      } else {
        back = nullptr;
      }
    }
  }

  if (forwok) {
    name = forwok->next_name;
  } else {
    name = "No name found";
  }

  if (forw == memh) {
    /* to be sure but this block is removed from the list */
    if (forwok) {
      if (backok) {
        forwok->next = (MemHead *)&backok->next;
        backok->prev = (MemHead *)&forwok->next;
        forwok->next_name = backok->name;
      } else {
        forwok->next = nullptr;
        mem_base->last = (LocalLink *)&forwok->next;
      }
    } else {
      if (backok) {
        backok->prev = nullptr;
        mem_base->first = &backok->next;
      } else {
        mem_base->first = mem_base->last = nullptr;
      }
    }
  } else {
    print_memory_error(name, "Additional error in header");
    return ("Additional error in header");
  }

  return name;
}

static void remove_link(volatile LocalListBase *list_base, void *vlink) {
  LocalLink *link = static_cast<LocalLink *>(vlink);

  if (link->next) {
    link->next->prev = link->prev;
  }
  if (link->prev) {
    link->prev->next = link->next;
  }

  if (list_base->last == link) {
    list_base->last = link->prev;
  }
  if (list_base->first == link) {
    list_base->first = link->next;
  }
}

static void remove_memblock(MemHead *memh) {
  mem_lock_thread();
  remove_link(mem_base, &memh->next);
  if (memh->prev) {
    if (memh->next) {
      MEMNEXT(memh->prev)->next_name = MEMNEXT(memh->next)->name;
    } else {
      MEMNEXT(memh->prev)->next_name = nullptr;
    }
  }
  mem_unlock_thread();

  atomic_sub_and_fetch_u(&tot_block, 1);
  atomic_sub_and_fetch_z(&mem_in_use, memh->len);
}

//-------------------------------------------------------------------------------------------------

void mem_guarded_free(void *mem, internal::AllocationType allocation_type) {
  MemTail *memt;
  MemHead *memh = static_cast<MemHead *>(mem);
  const char *name;

  if (memh == nullptr) {
    print_memory_error("free", "attempt to free nullptr");
    return;
  }

  if (sizeof(intptr_t) == 8) {
    if (intptr_t(memh) & 0x7) {
      print_memory_error("free", "attempt to free illegal pointer");
      return;
    }
  } else {
    if (intptr_t(memh) & 0x3) {
      print_memory_error("free", "attempt to free illegal pointer");
      return;
    }
  }

  memh--;

  if (allocation_type != internal::AllocationType::NEW_DELETE &&
      (memh->flag & MEMHEAD_FLAG_FROM_CPP_NEW) != 0)
  {
    report_error_on_address(
        mem, "Attempt to use C-style mem_free on a pointer created with CPP-style mem_new\n");
  }

  if (memh->tag1 == MEMFREE && memh->tag2 == MEMFREE) {
    print_memory_error(memh->name, "double free");
    return;
  }

  if ((memh->tag1 == MEMTAG1) && (memh->tag2 == MEMTAG2) && ((memh->len & 0x3) == 0)) {
    memt = (MemTail *)(((char *)memh) + sizeof(MemHead) + memh->len);
    if (memt->tag3 == MEMTAG3) {
      if (leak_detector_has_run) {
        print_memory_error(memh->name, free_after_leak_detection_message);
      }

      memh->tag1 = MEMFREE;
      memh->tag2 = MEMFREE;
      memt->tag3 = MEMFREE;
      remove_memblock(memh);

      return;
    }

    print_memory_error(memh->name, "end corrupt");
    name = check_memlist(memh);
    if (name != nullptr) {
      if (name != memh->name) {
        print_memory_error(name, "is also corrupt");
      }
    }
  } else {
    mem_lock_thread();
    name = check_memlist(memh);
    mem_unlock_thread();
    if (name == nullptr) {
      print_memory_error("free", "pointer not in memlist");
    } else {
      print_memory_error(name, "error in header");
    }
  }

  tot_block--;
}

void *mem_guarded_malloc(size_t len, const char *str) {
  MemHead *memh;

#ifdef WITH_MEM_VALGRIND
  TODO;
#endif
  len = SIZET_ALIGN_4(len);

  memh = (MemHead *)malloc(len + sizeof(MemHead) + sizeof(MemTail));

  if (LIKELY(memh)) {
    make_memhead_header(memh, len, str, internal::AllocationType::ALLOC_FREE);

    if (LIKELY(len)) {
      if (UNLIKELY(malloc_debug_memset)) {
        memset(memh + 1, 255, len);
      }
#ifdef WITH_MEM_VALGRIND
      TODO;
#endif
    }

#ifdef DEBUG_MEMCOUNTER
    TODO;
#endif
    return (++memh);
  }

  print_error("Malloc returns null: len=" SIZET_FORMAT " in %s, total " SIZET_FORMAT "\n",
              SIZET_ARG(len),
              str,
              mem_in_use);
  return nullptr;
}

void *mem_guarded_malloc_aligned(size_t len,
                                 size_t alignment,
                                 const char *str,
                                 internal::AllocationType allocation_type) {
  // Huge alignment values don't make sense and wouldn't fit into uint16_t
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

  MemHead *memh = (MemHead *)aligned_alloc(len + extra_padding + sizeof(MemHead) + sizeof(MemTail),
                                           alignment);

  if (LIKELY(memh)) {
    memh = (MemHead *)((char *)memh + extra_padding);

    make_memhead_header(memh, len, str, allocation_type);
    memh->alignment = uint16_t(alignment);
    if (LIKELY(len)) {
      if (UNLIKELY(malloc_debug_memset)) {
        memset(memh + 1, 255, len);
      }
#ifdef WITH_MEM_VALGRIND
      TODO;
#endif
    }

#ifdef DEBUG_MEMCOUNTER
    TODO;
#endif
    return (++memh);
  }

  print_error("aligned_malloc returns null: len=" SIZET_FORMAT " in %s, total " SIZET_FORMAT "\n",
              SIZET_ARG(len),
              str,
              mem_in_use);
  return nullptr;
}

static void *mem_guarded_malloc_array_aligned(
    size_t len, size_t size, size_t alignment, const char *str, size_t &r_bytes_num) {
  if (UNLIKELY(!mem_size_safe_multiply(len, size, &r_bytes_num))) {
    print_error(
        "Calloc array aborted due to integer overflow: "
        "len=" SIZET_FORMAT "x" SIZET_FORMAT " in %s, total " SIZET_FORMAT "\n",
        SIZET_ARG(len),
        SIZET_ARG(size),
        str,
        mem_in_use);
    abort();
    return nullptr;
  }

  if (alignment <= MEM_MIN_CPP_ALIGNMENT) {
    return mem_malloc(r_bytes_num, str);
  }

  todo();
  return nullptr;
  // return mem_malloc_aligned(r_bytes_num, alignment, str);
}

void *mem_guarded_malloc_array(size_t len, size_t size, const char *str) {
  size_t total_size;
  if (UNLIKELY(!mem_size_safe_multiply(len, size, &total_size))) {
    print_error(
        "Malloc array aborted due to integer overflow: "
        "len=" SIZET_FORMAT "x" SIZET_FORMAT " in %s, total " SIZET_FORMAT " \n",
        SIZET_ARG(len),
        SIZET_ARG(size),
        str,
        mem_in_use);
    abort();
    return nullptr;
  }

  return mem_guarded_malloc(total_size, str);
}

void *mem_guarded_calloc(size_t len, const char *str) {
  MemHead *memh;
  len = SIZET_ALIGN_4(len);

  memh = (MemHead *)calloc(len + sizeof(MemHead) + sizeof(MemTail), 1);
  if (LIKELY(memh)) {
    make_memhead_header(memh, len, str, internal::AllocationType::ALLOC_FREE);
#ifdef DEBUG_MEMCOUNTER
    TODO;
#endif
    return (++memh);
  }

  print_error("Calloc returns null: len=" SIZET_FORMAT " in %s, total " SIZET_FORMAT "\n",
              SIZET_ARG(len),
              str,
              mem_in_use);
  return nullptr;
}

void *mem_guarded_calloc_array_aligned(size_t len,
                                       size_t size,
                                       size_t alignment,
                                       const char *str) {
  size_t bytes_num;
  // There is no lower level calloc with an alignment parameter
  void *ptr = mem_guarded_malloc_array_aligned(len, size, alignment, str, bytes_num);
  if (!ptr) {
    return nullptr;
  }

  memset(ptr, 0, bytes_num);
  return ptr;
}

static const char mem_printmemlist_pydict_script[] =
    "mb_userinfo = {}\n"
    "totmem = 0\n"
    "for mb_item in membase:\n"
    "    mb_item_user_size = mb_userinfo.setdefault(mb_item['name'], [0,0])\n"
    "    mb_item_user_size[0] += 1 # Add a user\n"
    "    mb_item_user_size[1] += mb_item['len'] # Increment the size\n"
    "    totmem += mb_item['len']\n"
    "print('(membase) items:', len(membase), '| unique-names:',\n"
    "      len(mb_userinfo), '| total-mem:', totmem)\n"
    "mb_userinfo_sort = list(mb_userinfo.items())\n"
    "for sort_name, sort_func in (('size', lambda a: -a[1][1]),\n"
    "                             ('users', lambda a: -a[1][0]),\n"
    "                             ('name', lambda a: a[0])):\n"
    "    print('\\nSorting by:', sort_name)\n"
    "    mb_userinfo_sort.sort(key = sort_func)\n"
    "    for item in mb_userinfo_sort:\n"
    "        print('name:%%s, users:%%i, len:%%i' %%\n"
    "              (item[0], item[1][0], item[1][1]))\n";

/* Prints in python syntax for easy */
static void mem_guarded_print_memlist_internal(int pydict) {
  MemHead *membl;

  mem_lock_thread();

  membl = static_cast<MemHead *>(mem_base->first);
  if (membl) {
    membl = MEMNEXT(membl);
  }

  if (pydict) {
    print_error("# membase_debug.py\n");
    print_error("membase = [\n");
  }
  while (membl) {
    if (pydict) {
      print_error("    {'len':" SIZET_FORMAT
                  ", "
                  "'name':'''%s''', "
                  "'pointer':'%p'},\n",
                  SIZET_ARG(membl->len),
                  membl->name,
                  (void *)(membl + 1));
    } else {
#ifdef DEBUG_MEMCOUNTER
      print_error("%s len: " SIZET_FORMAT " %p, count: %d\n",
                  membl->name,
                  SIZET_ARG(membl->len),
                  membl + 1,
                  membl->_count);
#else
      print_error("%s len: " SIZET_FORMAT " - 0x%p\n",
                  membl->name,
                  SIZET_ARG(membl->len),
                  (void *)(membl + 1));
#endif

#ifdef DEBUG_BACKTRACE_EXECINFO
      print_memhead_backtrace(membl);
#elif defined(DEBUG_BACKTRACE) && defined(WITH_ASAN)
      __asan_describe_address(membl);
#endif
    }
    if (membl->next) {
      membl = MEMNEXT(membl->next);
    } else {
      break;
    }
  }
  if (pydict) {
    print_error("]\n\n");
    print_error(mem_printmemlist_pydict_script);
  }

  mem_unlock_thread();
}

void mem_guarded_print_memlist() {
  mem_guarded_print_memlist_internal(0);
}

void mem_guarded_set_error_callback(void (*func)(const char *)) {
  error_callback = func;
}

size_t mem_guarded_get_memory_in_use() {
  size_t _mem_in_use;

  mem_lock_thread();
  _mem_in_use = mem_in_use;
  mem_unlock_thread();

  return _mem_in_use;
}

uint32_t mem_guarded_get_memory_blocks_in_use() {
  uint32_t _tot_block;

  mem_lock_thread();
  _tot_block = tot_block;
  mem_unlock_thread();

  return _tot_block;
}

void mem_guarded_clear_memlist() {
  mem_base->first = mem_base->last = nullptr;
}
