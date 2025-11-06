#include "Console.hpp"

#include <cstdio>

void ConsoleStream::print(const char *sz) {
  fprintf(stdout, "%s", sz);
}

void ConsoleStream::flush() {
  fflush(stdout);
}
