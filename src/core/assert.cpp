#include "assert.hpp"

#include "core/system.hpp"
#include <cstdio>
#include <cstdlib>

void assert_print_pos(const char *file, int line, const char *function, const char *id) {
  fprintf(stderr, "fe_assert failed: %s:%d, %s(), at \'%s\'\n", file, line, function, id);
}

void assert_print_extra(const char *str) {
  fprintf(stderr, "  %s\n", str);
}

void assert_print_backtrace() {
#ifndef NDEBUG
  system_backtrace(stderr);
#endif
}

void assert_abort() {
  abort();
}

void assert_unreachable_print(const char *file, const int line, const char *function) {
  fprintf(stderr, "Code marked as unreachable has been executed. Please report this as a bug.\n");
  fprintf(stderr, "Error found at %s:%d in %s.\n", file, line, function);
}
