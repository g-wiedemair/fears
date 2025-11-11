#pragma once

#include "core/string.hpp"
#include "fecore/LogStream.hpp"

#include <cstdio>

/// A stream that outputs to a file
class LogFileStream : public LogStream {
 private:
  FILE *file_ = nullptr;
  String filename_;

 public:
  FECORE_API LogFileStream() = default;
  FECORE_API ~LogFileStream() override;

  FECORE_API bool open(const char *filename);
  FECORE_API void close();

  FECORE_API void print(const char *sz) override;
  FECORE_API void flush() override;
};
