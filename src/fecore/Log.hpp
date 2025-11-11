#pragma once

#include "core/utildefines.hpp"
#include "fecore/fecore_api.hpp"

struct LogContext;

enum LogLevel {
  LOG_LEVEL_FATAL = 0,
  LOG_LEVEL_ERROR = 1,
  LOG_LEVEL_WARN = 2,
  LOG_LEVEL_INFO = 3,
  LOG_LEVEL_DEBUG = 4,
  LOG_LEVEL_TRACE = 5,
  LOG_LEVEL_LEN,
};

/* Each logger ID has one of these */
struct LogType {
  LogType *next;
  char identifier[64];
  LogContext *ctx;
  LogLevel level;
};

struct LogRef {
  const char *identifier;
  LogType *type;
  LogRef *next;
};

namespace Log {

FECORE_API void init();
FECORE_API void exit();

FECORE_API void log_ref_init(LogRef *log_ref);

FECORE_API void output_use_timestamp(bool set);
FECORE_API void output_use_memory(bool set);
FECORE_API void output_use_source(bool set);
FECORE_API void output_use_basename(bool set);
FECORE_API void set_level(LogLevel level);
FECORE_API void fatal_fn_set(void (*fatal_fn)(void *file_handle));

FECORE_API void logf(const LogType *lg,
                     LogLevel level,
                     const char *file_line,
                     const char *fn,
                     const char *format,
                     ...);

}  // namespace Log

#define LOG_ENSURE(log_ref) \
  ((log_ref)->type ? (log_ref)->type : (Log::log_ref_init(log_ref), (log_ref)->type))

#define LOG_AT_LEVEL(log_ref, verbose_level, ...) \
  { \
    const LogType *_lg_ty = LOG_ENSURE(log_ref); \
    if (_lg_ty->level >= verbose_level) { \
      Log::logf(_lg_ty, verbose_level, __FILE__ ":" STRINGIFY(__LINE__), __func__, __VA_ARGS__); \
    } \
  } \
  ((void)0)

/* Log with format string */
#define LOG_FATAL(log_ref, ...) LOG_AT_LEVEL(log_ref, LOG_LEVEL_FATAL, __VA_ARGS__)
#define LOG_ERROR(log_ref, ...) LOG_AT_LEVEL(log_ref, LOG_LEVEL_ERROR, __VA_ARGS__)
#define LOG_WARN(log_ref, ...) LOG_AT_LEVEL(log_ref, LOG_LEVEL_WARN, __VA_ARGS__)
#define LOG_INFO(log_ref, ...) LOG_AT_LEVEL(log_ref, LOG_LEVEL_INFO, __VA_ARGS__)
#define LOG_DEBUG(log_ref, ...) LOG_AT_LEVEL(log_ref, LOG_LEVEL_DEBUG, __VA_ARGS__)
#define LOG_TRACE(log_ref, ...) LOG_AT_LEVEL(log_ref, LOG_LEVEL_TRACE, __VA_ARGS__)
