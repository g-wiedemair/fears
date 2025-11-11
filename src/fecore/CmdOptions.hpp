#pragma once

#include "core/sys_types.hpp"
#include "fecore/Log.hpp"

/// This structure stores the command line options
struct CmdOptions {

  bool bsplash;                    //< show splash screen
  bool bsilent;                    //< run in silent mode (no output to screen)
  LogLevel log_level;              //< log level
                                   //
  bool binteractive;               //< start Feap interactively
                                   //
  bool bdebug_memory;              //< Enable memory debugging
                                   //
  char config_filename[FILE_MAX];  //< config file to use
  char input_filename[FILE_MAX];   //< input script to run

  CmdOptions() {
    defaults();
  }

  void defaults() {
    bsplash = true;
    bsilent = false;

    binteractive = true;

    bdebug_memory = false;

    config_filename[0] = 0;
    input_filename[0] = 0;
  }
};
