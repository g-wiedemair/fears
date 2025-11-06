#pragma once

#include <cstdio>

void system_backtrace_with_os_info(FILE *fp, const void *os_info);
void system_backtrace(FILE *fp);
