#include "Command.hpp"

#include "fecore/feap_version.hpp"
#include "fenda/CommandManager.hpp"

int CmdHelp::run(int argc, char **argv) {
  CommandManager *cm = CommandManager::get_handle();
  int n = cm->size();
  if (n == 0) {
    return 0;
  }

  printf("\nCommand overview:\n");

  CommandManager::CmdIterator it = cm->begin();
  while (it != cm->end()) {
    const char *name = (*it)->get_name();
    size_t len = strlen(name);
    printf("\t%s ", name);
    while (len++ - 15 < 0)
      putchar('.');
    const char *desc = (*it++)->get_description();
    printf(" : %s\n", desc);
  }

  return 0;
}

int CmdVersion::run(int argc, char **argv) {
  const char *version = feap_version_string();

#ifndef NDEBUG
  fprintf(stderr, "\nFeap version %s (DEBUG)\n", version);
#else
  fprintf(stderr, "\nFeap version %s\n", version);
#endif

  return 0;
}
