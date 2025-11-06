#include "core/intern/memory_alloc_intern.hpp"
#include "core/memory.hpp"

#include <any>
#include <mutex>
#include <vector>

bool leak_detector_has_run = false;
char free_after_leak_detection_message[] =
    "Freeing memory after the leak detector has run. This can happen when using "
    "static variables in C++ that are defined outside of functions. To fix this "
    "error, use the 'construct on first use' idiom.";

namespace {

bool fail_on_memleak = false;

class MemLeakPrinter {
 public:
  ~MemLeakPrinter() {
    leak_detector_has_run = true;
    const uint32_t leaked_blocks = mem_get_memory_blocks_in_use();
    if (leaked_blocks == 0) {
      return;
    }

    const size_t mem_in_use = mem_get_memory_in_use();
    printf("Error: Not freed memory blocks: %u, total unfree memory %f MB\n",
           leaked_blocks,
           double(mem_in_use) / 1024 / 1024);
    mem_print_memlist();

    /* In guarded implementation, the fact that all allocated memory blocks are stored in the
     * static 'membase' listbase is enough for LSAN to not detect them as leaks. Clearing it solves
     * that issue. */
    mem_clear_memlist();

    if (fail_on_memleak) {
      abort();
    }
  }
};

}  // namespace

void mem_init_memleak_detection() {
  // Calling this ensures that the memory usage counters outlive the memory leak detection
  memory_usage_init();

  // Ensure that the static memleak data storage is initialized
  std::any any_data = std::make_any<int>(0);
  internal::add_memleak_data(any_data);

  static MemLeakPrinter printer;
}

void internal::add_memleak_data(std::any data) {
  static std::mutex mutex;
  static std::vector<std::any> data_vec;
  std::lock_guard<std::mutex> lock{mutex};
  data_vec.push_back(std::move(data));
}
