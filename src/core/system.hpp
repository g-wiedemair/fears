#pragma once

#include "core/core_api.hpp"

#include <cstdio>

void system_backtrace_with_os_info(FILE *fp, const void *os_info);
CORE_API void system_backtrace(FILE *fp);
