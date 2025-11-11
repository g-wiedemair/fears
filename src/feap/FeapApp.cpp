#include "FeapApp.hpp"

#include "core/memory.hpp"
#include "core/string.hpp"
#include "core/sys_types.hpp"
#include "feap/CommandManager.hpp"
#include "feap/Interruption.hpp"
#include "fecore/Console.hpp"
#include "fecore/FeKernel.hpp"
#include "fecore/Log.hpp"
#include "fecore/feap_version.hpp"
#include "fecore/utils.hpp"
#include "fenda/FeapModel.hpp"
#include "fenda/fenda.hpp"

static FeapApp *instance_ = nullptr;
static LogRef LOG = {"feap.app"};

FeapApp::~FeapApp() {
  Console *console = Console::get_handle();
  mem_delete(console);
}

FeapApp *FeapApp::init(int argc, char **argv) {
  if (instance_ == nullptr) {
    instance_ = mem_new<FeapApp>(__func__);
  }

  // parse the command line
  if (instance_->parse_cmd_line(argc, argv) == false) {
    mem_delete(instance_);
    return nullptr;
  }

  LOG_TRACE(&LOG, "Initializing FeKernel");
  FeKernel::init();

  LOG_TRACE(&LOG, "Initializing fenda library");
  fenda::init_library();

  // read the configuration file if specified
  if (instance_->ops_.config_filename[0]) {
    if (fenda::configure(instance_->ops_.config_filename, instance_->config_) == false) {
      LOG_ERROR(&LOG, "An error occurred reading the configuration file.");
      mem_delete(instance_);
      return nullptr;
    }
  }

  return instance_;
}

int FeapApp::run() {
  // activate interruption handler
  Interruption I;

  if (ops_.binteractive) {
    return prompt();
  } else {
    return run_model();
  }
}

FeapModel *FeapApp::get_current_model() {
  if (instance_)
    return instance_->fem_;
  else
    return nullptr;
}

int FeapApp::prompt() {
  Console *shell = Console::get_handle();
  const char *version = feap_version_string();
  shell->set_title("Feap ", version);
  printf("\nEntering feap interactive mode. Type 'help' for a list of commands.");

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
      Command *cmd = cm->find("help");
      if (cmd)
        cmd->run(0, nullptr);
    }
  }

  mem_delete(cm);
}

int FeapApp::run_model() {
  // create a new model
  FeapModel fem;
  this->set_current_model(&fem);

  // add console stream to log file
  if (!ops_.bsilent) {
    fem.get_logfile().set_logstream(mem_new<ConsoleStream>(__FILE__));
  }

  // reset the model
  this->set_current_model(nullptr);
  return 0;
}

void FeapApp::set_current_model(FeapModel *fem) {
  fem_ = fem;
}

void FeapApp::finish() {
  FeKernel::shutdown();

  mem_delete(instance_);
  instance_ = nullptr;
}

bool FeapApp::parse_cmd_line(int argc, char **argv) {
  CmdOptions &ops = ops_;

  // Set initial configuration file
  if (ops.config_filename[0] == 0) {
    char path[FILE_MAX] = {0};
    utils::get_app_path(path, FILE_MAX - 1);
    fsnprintf(ops.config_filename, sizeof(ops.config_filename), "%sfeap.config", path);
  }

  // say hello
  ConsoleStream s;
  if (argc == 1 || (strcmp(argv[1], "--no-splash") != 0 && strcmp(argv[1], "--silent") != 0 &&
                    strcmp(argv[1], "-s") != 0))
  {
    fenda::say_hello(s);
  }

  // loop over the arguments
  for (int i = 1; i < argc; ++i) {
    char *arg = argv[i];
    if (STR_ELEM(arg, "-h", "--help")) {
      this->print_help();
      return false;

    } else if (STR_ELEM(arg, "--no-splash")) {
      ops_.bsplash = false;

    } else if (STR_ELEM(arg, "-s", "--silent")) {
      ops_.bsilent = true;

    } else if (STR_ELEM(arg, "-v", "--version")) {
      printf("Feap version %s\n", feap_version_string());

    } else if (STR_ELEM(arg, "-i", "--interactive")) {
      ops_.binteractive = true;

    } else if (STR_ELEM(arg, "-d", "--debug-all")) {
      ops_.bdebug_memory = true;

    } else if (STR_ELEM(arg, "--debug-memory")) {
      ops_.bdebug_memory = true;

    } else if (STR_ELEM(arg, "-l", "--log-level")) {
      ++i;
      if (STR_ELEM(argv[i], "trace")) {
        ops_.log_level = LOG_LEVEL_TRACE;
      } else if (STR_ELEM(argv[i], "debug")) {
        ops_.log_level = LOG_LEVEL_DEBUG;
      } else if (STR_ELEM(argv[i], "info")) {
        ops_.log_level = LOG_LEVEL_INFO;
      } else {
        LOG_ERROR(&LOG, "Invalid log level: %s", argv[i]);
        ops_.log_level = LOG_LEVEL_WARN;
        return false;
      }
      Log::set_level(ops_.log_level);

    } else if (STR_ELEM(arg, "-f", "--input-file")) {
      ++i;
      const char *ext = strrchr(argv[i], '.');
      if (ext == nullptr) {
        // We assume a default extension of .fea
        snprintf(ops_.input_filename, sizeof(ops_.input_filename), "%s.fea", argv[i]);
      } else {
        strcpy(ops_.input_filename, argv[i]);
      }
      ops_.binteractive = false;
      return true;
    } else {
      // If no input file is given yet, we'll assume this is the input file
      if (ops_.input_filename[0] == 0) {
        const char *ext = strrchr(argv[i], '.');
        if (ext == nullptr) {
          // We assume a default extension of .fea
          snprintf(ops_.input_filename, sizeof(ops_.input_filename), "%s.fea", argv[i]);
        } else {
          strcpy(ops_.input_filename, argv[i]);
        }
        ops_.binteractive = false;
        return true;
      } else {
        LOG_ERROR(&LOG, "Invalid command line option: %s", arg);
        return false;
      }
    }
  }

  this->print_help();
  return true;
}

void FeapApp::print_help() {
  printf("Usage: feap [options]\n");
  printf("\t-h | --help .......: Show this help message\n");
  printf("\t   | --no-splash ..: Don't show the splash screen\n");
  printf("\t-s | --silent .....: Don't show any output\n");
  printf("\t-v | --version ....: Show version information\n");
  printf("\t-i | --interactive : Start feap in interactive mode\n");
  printf("\t-d | --debug-all ..: Enable debug mode\n");
  printf("\t   | --debug-memory: Enable memory debugging\n");
  printf("\t-l | --log-level ..: Set log level [trace,debug,info]\n");
  printf("\t-f | --input-file .: Run a feap script [script.fea]\n");
}

FeapApp *FeapApp::get_handle() {
  return instance_;
}
