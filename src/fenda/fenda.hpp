#pragma once

#include "fenda/fenda_api.hpp"

class LogStream;

namespace fenda {

FENDA_API void say_hello(LogStream &log);

FENDA_API void init_library();

}  // namespace fenda
