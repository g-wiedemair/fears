#include "Log.hpp"

#include "core/assert.hpp"
#include "core/memory.hpp"

#include <cassert>
#include <mutex>

#if defined(_MSC_VER)
#  ifndef NOMINMAX
#    define NOMINMAX
#  endif
#  include <Windows.h>

#  include <VersionHelpers.h> /* This needs to be included after Windows.h. */
#  include <io.h>
#  if !defined(ENABLE_VIRTUAL_TERMINAL_PROCESSING)
#    define ENABLE_VIRTUAL_TERMINAL_PROCESSING 0x0004
#  endif
#endif

static LogContext *g_ctx = nullptr;

static std::mutex LOG_MUTEX;

#define LOG_FILTER_COUNT 2

struct IDFilter {
  IDFilter *next;
  // Over alloc
  char match[0];
};

struct LogContext {
  // Single linked list of types
  LogType *types;
  // Single linked list of references
  LogRef *refs;
#ifdef WITH_LOG_PTHREADS
  TODO;
#endif

  // exclude, include filters
  IDFilter *filters[LOG_FILTER_COUNT];
  bool use_color;
  bool use_source;
  bool use_basename;
  bool use_timestamp;
  bool use_memory;

  int output;
  FILE *output_file;

  uint64_t timestamp_tick_start;

  struct {
    LogLevel level;
  } default_type;

  struct {
    void (*error_fn)(void *file_handle);
    void (*fatal_fn)(void *file_handle);
    void (*backtrace_fn)(void *file_handle);
  } callbacks;
};

#define LOG_BUF_LEN_INIT 512

struct LogStringBuf {
  char *data;
  uint32_t len;
  uint32_t len_alloc;
  bool is_alloc;
};

enum LogColor {
  COLOR_DEFAULT,
  COLOR_RED,
  COLOR_GREEN,
  COLOR_YELLOW,
  COLOR_DIM,
  COLOR_RESET,
  COLOR_LEN,
};

static const char *color_table[COLOR_LEN] = {nullptr};

static void color_table_init(bool use_color) {
  for (int i = 0; i < COLOR_LEN; i++) {
    color_table[i] = "";
  }
  if (use_color) {
    color_table[COLOR_DEFAULT] = "\033[1;37m";
    color_table[COLOR_RED] = "\033[1;31m";
    color_table[COLOR_GREEN] = "\033[1;32m";
    color_table[COLOR_YELLOW] = "\033[1;33m";
    color_table[COLOR_DIM] = "\033[2;37m";
    color_table[COLOR_RESET] = "\033[0m";
  }
}

static LogColor log_level_to_color(LogLevel level) {
  switch (level) {
    case LOG_LEVEL_FATAL:
    case LOG_LEVEL_ERROR:
      return COLOR_RED;
    case LOG_LEVEL_WARN:
      return COLOR_YELLOW;
    case LOG_LEVEL_INFO:
    case LOG_LEVEL_DEBUG:
    case LOG_LEVEL_TRACE:
      return COLOR_DEFAULT;
  }
  assert(false);
  return COLOR_DEFAULT;
}

static const char *log_level_as_text(LogLevel level) {
  switch (level) {
    case LOG_LEVEL_FATAL:
      return "FATAL";
    case LOG_LEVEL_ERROR:
      return "ERROR";
    case LOG_LEVEL_WARN:
      return "WARNING";
    case LOG_LEVEL_INFO:
      return "INFO";
    case LOG_LEVEL_DEBUG:
      return "DEBUG";
    case LOG_LEVEL_TRACE:
      return "TRACE";
  }

  return "INVLAID_LEVEL";
}

#ifdef _WIN32
static DWORD log_previous_console_mode = 0;
#endif

static void ctx_output_set(LogContext *ctx, void *file_handle) {
  ctx->output_file = static_cast<FILE *>(file_handle);
  ctx->output = fileno(ctx->output_file);
#if defined(__unix__) || defined(__APPLE__)
  TODO;
#elif defined(WIN32)
  GetConsoleMode(GetStdHandle(STD_OUTPUT_HANDLE), &log_previous_console_mode);

  ctx->use_color = false;
  if (IsWindows10OrGreater() && isatty(ctx->output)) {
    DWORD mode = log_previous_console_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING;
    if (SetConsoleMode(GetStdHandle(STD_OUTPUT_HANDLE), mode)) {
      ctx->use_color = true;
    }
  }
#endif
}

static LogContext *ctx_init() {
  LogContext *ctx = mem_calloc<LogContext>(__func__);
#ifdef WITH_LOG_PTHREADS
  TODO;
#endif
  ctx->default_type.level = LOG_LEVEL_WARN;
  ctx->use_source = true;
  ctx_output_set(ctx, stdout);

  return ctx;
}

static void ctx_free(LogContext *ctx) {
#if defined(WIN32)
  SetConsoleMode(GetStdHandle(STD_OUTPUT_HANDLE), log_previous_console_mode);
#endif

  while (ctx->types != nullptr) {
    LogType *item = ctx->types;
    ctx->types = item->next;
    mem_free(item);
  }

  while (ctx->refs != nullptr) {
    LogRef *item = ctx->refs;
    ctx->refs = item->next;
    item->type = nullptr;
  }

  for (uint32_t i = 0; i < LOG_FILTER_COUNT; i++) {
    while (ctx->filters[i] != nullptr) {
      todo();
    }
  }

#ifdef WITH_LOG_PTHREADS
  TODO;
#endif

  mem_free(ctx);
}

void Log::init() {
  g_ctx = ctx_init();
  color_table_init(g_ctx->use_color);
}

void Log::exit() {
  ctx_free(g_ctx);
}

static LogType *log_ctx_type_find_by_name(LogContext *ctx, const char *identifier) {
  for (LogType *ty = ctx->types; ty; ty = ty->next) {
    if (STREQ(identifier, ty->identifier)) {
      return ty;
    }
  }
  return nullptr;
}

/**
 * Filter the identifier based on very basic globbing
 * - 'foo' matches everything starting with 'foo'
 * - '*bar*' match for 'foo.bar' & 'baz.bar' & 'foo.barbaz'
 * - '*' matches everything
 */
static bool log_ctx_filter_check(LogContext *ctx, const char *identifier) {
  if (ctx->filters[0] == nullptr && ctx->filters[1] == nullptr &&
      ctx->default_type.level >= LOG_LEVEL_INFO)
  {
    // No filters but level specified? match everything
    return true;
  }

  const size_t identifier_len = strlen(identifier);
  for (uint32_t i = 0; i < LOG_FILTER_COUNT; ++i) {
    const IDFilter *flt = ctx->filters[i];
    while (flt != nullptr) {
      todo();
    }
  }

  return false;
}

static LogType *log_ctx_type_register(LogContext *ctx, const char *identifier) {
  assert(log_ctx_type_find_by_name(ctx, identifier) == nullptr);

  LogType *ty = mem_calloc<LogType>(__func__);
  ty->next = ctx->types;
  ctx->types = ty;
  strncpy(ty->identifier, identifier, sizeof(ty->identifier) - 1);
  ty->ctx = ctx;

  if (log_ctx_filter_check(ctx, ty->identifier)) {
    todo();
  } else {
    ty->level = std::min(ctx->default_type.level, LOG_LEVEL_WARN);
  }

  return ty;
}

static void log_ctx_error_action(LogContext *ctx) {
  if (ctx->callbacks.error_fn != nullptr) {
    ctx->callbacks.error_fn(ctx->output_file);
  }
}

static void log_ctx_fatal_action(LogContext *ctx) {
  if (ctx->callbacks.fatal_fn != nullptr) {
    ctx->callbacks.fatal_fn(ctx->output_file);
  }
  fflush(ctx->output_file);
  abort();
}

void Log::log_ref_init(LogRef *log_ref) {
#ifdef WITH_LOG_PTHREADS
  TODO;
#endif

  if (log_ref->type == nullptr) {
    log_ref->next = g_ctx->refs;
    g_ctx->refs = log_ref;

    LogType *log_ty = log_ctx_type_find_by_name(g_ctx, log_ref->identifier);
    if (log_ty == nullptr) {
      log_ty = log_ctx_type_register(g_ctx, log_ref->identifier);
    }

#ifdef WITH_LOG_PTHREADS
    TODO;
#else
    log_ref->type = log_ty;
#endif
  }

#ifdef WITH_LOG_PTHREADS
  TODO;
#endif
}

static void log_str_init(LogStringBuf *cstr, char *buf_stack, uint32_t buf_stack_len) {
  cstr->data = buf_stack;
  cstr->len_alloc = buf_stack_len;
  cstr->len = 0;
  cstr->is_alloc = false;
}

static void log_str_reserve(LogStringBuf *cstr, const uint32_t len) {
  if (len > cstr->len_alloc) {
    cstr->len_alloc *= 2;
    cstr->len_alloc = std::max(len, cstr->len_alloc);

    if (cstr->is_alloc) {
      todo();
    } else {
      todo();
      // char *data = mem_malloc_array<char>(cstr->len_alloc, __func__);
      // memcpy(data, cstr->data, cstr->len);
      // cstr->data = data;
      // cstr->is_alloc = true;
    }
  }
}

static void log_str_free(LogStringBuf *cstr) {
  if (cstr->is_alloc) {
    mem_free(cstr->data);
  }
}

static void log_str_append_with_len(LogStringBuf *cstr, const char *str, const uint32_t len) {
  uint32_t len_next = cstr->len + len;
  log_str_reserve(cstr, len_next);
  char *str_dst = cstr->data + cstr->len;
  memcpy(str_dst, str, len);
  cstr->len = len_next;
}

static void log_str_append(LogStringBuf *cstr, const char *str) {
  log_str_append_with_len(cstr, str, strlen(str));
}

static void log_str_append_char(LogStringBuf *cstr, const char c, const uint32_t len) {
  uint32_t len_next = cstr->len + len;
  log_str_reserve(cstr, len_next);
  char *str_dst = cstr->data + cstr->len;
  memset(str_dst, c, len);
  cstr->len = len_next;
}

static void log_str_vappendf(LogStringBuf *cstr, const char *format, va_list args) {
  const uint32_t len_max = 65535;
  uint32_t len_avail = cstr->len_alloc - cstr->len;
  while (true) {
    va_list args_copy;
    va_copy(args_copy, args);
    int retval = vsnprintf(cstr->data + cstr->len, len_avail, format, args_copy);
    va_end(args_copy);

    if (retval < 0) {
      // Some encoding error happend
      break;
    }

    if ((uint32_t)retval <= len_avail) {
      // Copy was successful
      cstr->len += (uint32_t)retval;
      break;
    }

    uint32_t len_alloc = cstr->len + (uint32_t)retval;
    if (len_alloc >= len_max) {
      // Safe upper-limit
      break;
    }

    log_str_reserve(cstr, len_alloc);
    len_avail = cstr->len_alloc - cstr->len;
  }
}

static void log_str_indent_multiline(LogStringBuf *cstr, const uint32_t indent_len) {
  // If there are multiple lines, indent them the same as the first for readability
  if (indent_len < 2) {
    return;
  }

  uint32_t num_newlines = 0;
  for (uint32_t i = 0; i < cstr->len; i++) {
    if (cstr->data[i] == '\n') {
      num_newlines++;
    }
  }
  if (num_newlines == 0) {
    return;
  }

  todo();
}

static void write_level(LogStringBuf *cstr, LogLevel level, bool use_color) {
  if (level >= LOG_LEVEL_INFO) {
    return;
  }

  if (use_color) {
    LogColor color = log_level_to_color(level);
    log_str_append(cstr, color_table[color]);
    log_str_append(cstr, log_level_as_text(level));
    log_str_append(cstr, color_table[COLOR_RESET]);
  } else {
    log_str_append(cstr, log_level_as_text(level));
  }

  log_str_append(cstr, " ");
}

static void write_type(LogStringBuf *cstr, const LogType *lg) {
  const uint32_t len = strlen(lg->identifier);
  log_str_append_with_len(cstr, lg->identifier, len);

  const uint32_t type_align_width = 16;
  const uint32_t num_spaces = (len < type_align_width) ? type_align_width - len : 0;
  log_str_append_char(cstr, ' ', num_spaces + 1);
}

void Log::logf(const LogType *lg,
               LogLevel level,
               const char *file_line,
               const char *fn,
               const char *format,
               ...) {
  LogStringBuf cstr;
  char cstr_stack_buf[LOG_BUF_LEN_INIT];
  log_str_init(&cstr, cstr_stack_buf, sizeof(cstr_stack_buf));

  if (lg->ctx->use_timestamp) {
    todo();
  }
  if (lg->ctx->use_memory) {
    todo();
  }
  write_type(&cstr, lg);

  log_str_append(&cstr, "| ");

  const uint64_t multiline_ident_len = cstr.len;

  write_level(&cstr, level, lg->ctx->use_color);

  {
    va_list ap;
    va_start(ap, format);
    log_str_vappendf(&cstr, format, ap);
    va_end(ap);
  }

  log_str_indent_multiline(&cstr, multiline_ident_len);

  log_str_append(&cstr, "\n");

  /* Output could be optional */
  {
    std::scoped_lock lock(LOG_MUTEX);
    int bytes_written = write(lg->ctx->output, cstr.data, cstr.len);
    (void)bytes_written;
  }

  log_str_free(&cstr);

  if (lg->ctx->callbacks.backtrace_fn) {
    todo();
  }

  if (level == LOG_LEVEL_ERROR) {
    log_ctx_error_action(lg->ctx);
  }

  if (level == LOG_LEVEL_FATAL) {
    log_ctx_fatal_action(lg->ctx);
  }
}
