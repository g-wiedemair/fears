#include "system.hpp"

#include "core/mutex.hpp"

void system_backtrace(FILE *fp) {
  static Mutex mutex;
  std::scoped_lock lock(mutex);
  system_backtrace_with_os_info(fp, nullptr);
}
