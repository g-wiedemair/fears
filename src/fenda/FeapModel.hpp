#pragma once

#include "femech/FeMechModel.hpp"

/// The Feap model specializes the FEModel class to implement specific functionality
/// In addition it adds support for all I/O capabilities
class FeapModel : public FeMechModel {
 private:
  String title_;
  String input_file_;
  String log_file_;

  LogFile log_;

  bool becho_ = true;  //< echo input to logfile

 public:
};
