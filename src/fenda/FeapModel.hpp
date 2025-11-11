#pragma once

#include "fecore/Log.hpp"
#include "femech/FeMechModel.hpp"
#include "fenda/LogFile.hpp"

/// The Feap model specializes the FEModel class to implement specific functionality
/// In addition it adds support for all I/O capabilities
class FeapModel : public FeMechModel {
 private:
  const char *title_;
  const char *input_filename_;
  const char *log_filename_;

  LogLevel log_level_;
  LogFile logfile_;

  bool becho_ = true;  //< echo input to logfile

 public:
  FENDA_API FeapModel();
  FENDA_API ~FeapModel() override = default;

  FENDA_API void set_title(const char *title) {
    title_ = title;
  }
  FENDA_API void set_input_filename(const char *filename) {
    input_filename_ = filename;
  }
  FENDA_API void set_log_filename(const char *filename) {
    log_filename_ = filename;
  }
  FENDA_API void set_log_level(LogLevel level) {
    log_level_ = level;
  }

  FENDA_API LogFile &get_logfile();

  FENDA_API bool read_input_file(const char *filename);
};
