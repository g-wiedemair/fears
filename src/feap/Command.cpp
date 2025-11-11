#include "Command.hpp"

#include "feap/CommandManager.hpp"
#include "feap/FeapApp.hpp"
#include "fecore/Console.hpp"
#include "fecore/feap_version.hpp"
#include "fenda/FeapModel.hpp"

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
    while (int(len++) - 15 < 0) {
      putchar('.');
    }
    const char *desc = (*it++)->get_description();
    printf(" : %s\n", desc);
  }

  return 0;
}

int CmdQuit::run(int argc, char **argv) {
  return 1;
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

int CmdRun::run(int argc, char **argv) {
  FeapApp *app = FeapApp::get_handle();
  fassert(app);

  FeapModel *fem = app->get_current_model();
  if (fem) {
    fprintf(stderr,
            "A model is running. You must stop the active model before running this command.\n");
    return 0;
  }

  if (app->parse_cmd_line(argc, argv) == false)
    return 0;

  app->run_model();
  Console *shell = Console::get_handle();
  shell->set_title("Feap ", feap_version_string());

  return 0;
}
