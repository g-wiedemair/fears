#pragma once

#include "fecore/LogFileStream.hpp"
#include "fenda/fenda_api.hpp"

#include <cstdint>

/// Class used for logging purposes
/// This class can output to different files at the same time.
class FENDA_API LogFile {
 public:
  enum Mode : uint8_t {
    LOG_NEVER = 0b00,
    LOG_FILE = 0b01,
    LOG_CONSOLE = 0b10,
    LOG_FILE_AND_CONSOLE = LOG_FILE | LOG_CONSOLE,
  };

 public:
  LogFile();
  LogFile(const LogFile &) = delete;
  LogFile(LogFile &&) = delete;
  ~LogFile();

  bool open(const char *filename);
  void close();

  inline void set_logstream(LogStream *stream) {
    console_ = stream;
  }

 private:
  LogStream *console_;
  LogFileStream *file_;
  Mode mode_;
};
