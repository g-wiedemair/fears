#include "FeapApp.hpp"

#include "core/memory.hpp"
#include "core/string.hpp"
#include <cstdio>

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

  // create the feap app
  FeapApp app;

  // initialize the app
  if (app.init(argc, argv) == false)
    return 1;

  // TODO:

  app.finish();

  return 0;
}
