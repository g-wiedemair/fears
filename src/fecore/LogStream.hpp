#pragma once

#include "fecore/fecore_api.hpp"

/// Class used to create an abstract interface to a stream
class FECORE_API LogStream {
 public:
  LogStream() = default;
  virtual ~LogStream() = default;

  virtual void print(const char *sz) = 0;

  void printf(const char *sz, ...);

  virtual void flush() {}
};
