#include "LogStream.hpp"

#include "core/string.hpp"
#include <cstdarg>

void LogStream::printf(const char *sz, ...) {
  va_list args;

  char txt[1024] = {0};
  va_start(args, sz);
  fvsnprintf(txt, sizeof(txt), sz, args);
  va_end(args);

  print(txt);
}
