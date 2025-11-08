#include "fenda.hpp"

#include "core/string.hpp"
#include "fecore/LogStream.hpp"
#include "fecore/feap_version.hpp"

void fenda::say_hello(LogStream &log) {
  char version[128] = {0};
  const char *version_string = feap_version_string();
  fsnprintf(version, sizeof(version), "  version %s\n", version_string);

  log.print("===========================================================================\n");
  log.print("  Feap\n");
  log.print("  Finite Elements Application\n");
  log.print(version);
  log.print("===========================================================================\n");
  log.print("\n");
}
