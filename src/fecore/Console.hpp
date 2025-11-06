#pragma once

#include "fecore/LogStream.hpp"
#include "fecore/fecore_api.hpp"

class Console {};

//-------------------------------------------------------------------------------------------------

class FECORE_API ConsoleStream : public LogStream {
 public:
  void print(const char *sz) override;

  void flush() override;
};
