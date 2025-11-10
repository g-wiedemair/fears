#include "fenda.hpp"

#include "core/string.hpp"
#include "fecore/LogStream.hpp"
#include "fecore/feap_version.hpp"

void fenda::say_hello(LogStream &log) {
  char version[128] = {0};
  const char *version_string = feap_version_string();
  fsnprintf(version, sizeof(version), "  version %s\n", version_string);

  log.print("===========================================================================\n");
  log.print("         ________    _________    ___________     ___________              \n");
  log.print("        |        |\\ |        |\\  |           |\\  |           |\\        \n");
  log.print("        |    ____|| |    ____||  |    ___    ||  |    ___    ||            \n");
  log.print("        |   |\\___\\| |   |\\___\\|  |   |\\__|   ||  |   |\\__|   ||      \n");
  log.print("        |   ||__    |   ||__     |   ||__|   ||  |   ||__|   ||            \n");
  log.print("        |       |\\  |       |\\   |           ||  |           ||          \n");
  log.print("        |    ___||  |    ___||   |    ___    ||  |    _______||            \n");
  log.print("        |   |\\__\\|  |   |\\__\\|   |   |\\__|   ||  |   |\\______\\|     \n");
  log.print("        |   ||      |   ||___    |   ||  |   ||  |   ||                    \n");
  log.print("        |   ||      |        |\\  |   ||  |   ||  |   ||                   \n");
  log.print("        |___||      |________||  |___||  |___||  |___||                    \n");
  log.print("                                                                           \n");
  log.print("        F I N I T E   E L E M E N T S   A P P L I C A T I O N              \n");
  log.print("                                                                           \n");
  log.print(version);
  log.print("  copyright (c) 2025-now - All rights reserved                             \n");
  log.print("                                                                           \n");
  log.print("===========================================================================\n");
  log.print("\n");
}
