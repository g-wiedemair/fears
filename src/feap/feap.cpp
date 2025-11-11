#include "FeapApp.hpp"

#include "core/memory.hpp"
#include "core/string.hpp"
#include "core/system.hpp"
#include "fecore/Log.hpp"

#include <cstdio>

static void callback_log_fatal(void *fp) {
  system_backtrace(static_cast<FILE *>(fp));
}

int main(int argc, char **argv) {
  // Guarded allocator
  {
    for (int i = 1; i < argc; ++i) {
      if (STR_ELEM(argv[i], "-d", "--debug", "--debug-memory", "--debug-all")) {
        printf("Switching to fully guarded memory allocator.\n");
        mem_use_guarded_allocator();
        break;
      }
    }
    mem_init_memleak_detection();
  }

  // Initialize logging
  Log::init();
  Log::output_use_timestamp(true);
  Log::fatal_fn_set(callback_log_fatal);

  // initialize the app
  FeapApp *app = FeapApp::init(argc, argv);
  if (app == nullptr) {
    Log::exit();
    return 1;
  }

  int nret = app->run();

  // cleanup
  app->finish();
  Log::exit();

  return nret;
}
