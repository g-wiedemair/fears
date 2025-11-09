#pragma once

#include "fenda/fenda_api.hpp"

class LogStream;
class FeapConfig;

namespace fenda {

/// Print hello message
FENDA_API void say_hello(LogStream &log);

/// Initialize all the fenda modules
FENDA_API void init_library();

/// Read the configuration file
FENDA_API bool configure(const char *file, FeapConfig &config);

}  // namespace fenda
