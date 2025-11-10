#include "FeapApp.hpp"

#include "core/memory.hpp"
#include "core/string.hpp"
#include "core/sys_types.hpp"
#include "fecore/Console.hpp"
#include "fecore/FeKernel.hpp"
#include "fecore/Log.hpp"
#include "fecore/feap_version.hpp"
#include "fecore/utils.hpp"
#include "fenda/CommandManager.hpp"
#include "fenda/Interruption.hpp"
#include "fenda/fenda.hpp"

static LogRef LOG = {"feap.app"};

FeapApp::FeapApp() {}

FeapApp::~FeapApp() {
  Console *console = Console::get_handle();
  mem_delete(console);
}

bool FeapApp::init(int argc, char **argv) {
  LOG_TRACE(&LOG, "Initializing FeKernel");
  FeKernel::init();

  // parse the command line
  if (parse_cmd_line(argc, argv) == false)
    return false;

  // say hello
  ConsoleStream s;
  if (_ops.bsplash && !_ops.bsilent) {
    fenda::say_hello(s);
  }

  LOG_TRACE(&LOG, "Initializing fenda library");
  fenda::init_library();

  // read the configuration file if specified
  if (_ops.config_filename[0]) {
    if (fenda::configure(_ops.config_filename, _config) == false) {
      LOG_FATAL(&LOG, "An error occurred reading the configuration file.");
      return false;
    }
  }

  return true;
}

int FeapApp::run() {
  // activate interruption handler
  Interruption I;

  if (_ops.binteractive) {
    return prompt();
  } else {
    return run_model();
  }
}

int FeapApp::prompt() {
  Console *shell = Console::get_handle();
  const char *version = feap_version_string();
  shell->set_title("Feap ", version);

  process_commands();

  return 0;
}

void FeapApp::process_commands() {
  Console *shell = Console::get_handle();

  CommandManager *cm = CommandManager::get_handle();

  int argc = 0;
  char *argv[32];
  while (true) {
    shell->get_command(argc, argv);
    if (argc > 0) {
      Command *cmd = cm->find(argv[0]);
      if (cmd) {
        int nret = cmd->run(argc, argv);
        if (nret == 1)
          break;
      } else
        printf("Unknown command: %s\n", argv[0]);
    } else {
      break;
    }
  }

  mem_delete(cm);
}

int FeapApp::run_model() {
  todo();
  return 0;
}

void FeapApp::finish() {
  FeKernel::shutdown();
}

bool FeapApp::parse_cmd_line(int argc, char **argv) {
  CmdOptions &ops = _ops;

  // Set initial configuration file
  if (ops.config_filename[0] == 0) {
    char path[FILE_MAX] = {0};
    utils::get_app_path(path, FILE_MAX - 1);
    fsnprintf(ops.config_filename, sizeof(ops.config_filename), "%sfeap.config", path);
  }

  // loop over the arguments
  // TODO:

  return true;
}
