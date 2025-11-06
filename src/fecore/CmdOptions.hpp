#pragma once

#include "core/sys_types.hpp"

/// This structure stores the command line options
struct CmdOptions {

  bool bsplash;  //< show splash screen
  bool bsilent;  //< run in silent mode (no output to screen)

  char config_filename[FILE_MAX];

  CmdOptions() {
    defaults();
  }

  void defaults() {
    bsplash = true;
    bsilent = false;

    config_filename[0] = 0;
  }
};
