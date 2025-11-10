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
  ~FeapApp();

  bool init(int argc, char **argv);
  int run();
  void finish();

 private:
  /// Show Feap prompt
  int prompt();
  void process_commands();

  /// Run a Feap model
  int run_model();

  bool parse_cmd_line(int argc, char **argv);

 public:
  FeapApp(const FeapApp &) = delete;
  FeapApp &operator=(const FeapApp &) = delete;
};
