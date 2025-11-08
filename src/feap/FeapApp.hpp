//
// Created by wig on 05.11.2025.
//

#pragma once

#include "fecore/CmdOptions.hpp"
#include "fenda/FeapConfig.hpp"

class FeapApp {
 private:
  CmdOptions _ops;     // command line options
  FeapConfig _config;  // configuration options
                       //
                       // FeapModel *_fem;     // current model

 public:
  FeapApp();

  bool init(int argc, char **argv);
  void finish();

 private:
  bool parse_cmd_line(int argc, char **argv);

 public:
  FeapApp(const FeapApp &) = delete;
  FeapApp &operator=(const FeapApp &) = delete;
};
