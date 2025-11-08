#include "FeapApp.hpp"

#include "core/string.hpp"
#include "core/sys_types.hpp"
#include "fecore/Console.hpp"
#include "fecore/FeKernel.hpp"
#include "fecore/Log.hpp"
#include "fecore/utils.hpp"
#include "fenda/fenda.hpp"

#include <cstdio>

static LogRef LOG = {"feap.app"};

FeapApp::FeapApp() {}

bool FeapApp::init(int argc, char **argv) {
  // initialize kernel
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
    // if (fenda::configure(_ops.config_filename, _config) == false) {
    //   fprintf(stderr, "FATAL ERROR: An error occurred reading the configuration file.\n");
    //   return false;
    // }
  }

  return true;
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
