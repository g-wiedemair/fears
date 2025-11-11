#pragma once

#include "fecore/CmdOptions.hpp"
#include "fenda/FeapConfig.hpp"

class FeapModel;

class FeapApp {
 private:
  CmdOptions ops_;     // command line options
  FeapConfig config_;  // configuration options
                       //
  FeapModel *fem_;     // current model

 public:
  ~FeapApp();

  static FeapApp *init(int argc, char **argv);
  int run();
  void finish();

 private:
  FeapApp() = default;

  /// Show Feap prompt
  int prompt();
  void process_commands();

  /// Run a Feap model
  int run_model();
  void set_current_model(FeapModel *fem);

  bool parse_cmd_line(int argc, char **argv);
  void print_help();

 public:
  FeapApp(const FeapApp &) = delete;
  FeapApp &operator=(const FeapApp &) = delete;

 private:
  friend class CmdRun;
  static FeapApp *get_handle();

  /// Get the current model
  static FeapModel *get_current_model();

  template<typename T, typename... Args> friend T *mem_new(const char *, Args &&...);
};
