#pragma once

#include "core/core_api.hpp"
#include "core/defines_variadic.hpp"
#include <cstdarg>
#include <string>

_EXPORT_STD using String = std::string;

/**
 * String debugging
 */
#ifdef WITH_STRSIZE_DEBUG
TODO;
#else
#  define string_debug_size(str, str_maxncpy) \
    if (0) { \
      (void)str, (void)str_maxncpy; \
    } \
    ((void)0)
#endif

/**
 * Portable replacement for `snprintf`.
 */
CORE_API size_t fsnprintf(char *__restrict dst,
                          size_t dst_maxncpy,
                          const char *__restrict format,
                          ...);

/**
 * Portable replacement for `vsnprintf`.
 */
CORE_API size_t fvsnprintf(char *__restrict dst,
                           size_t dst_maxncpy,
                           const char *__restrict format,
                           va_list arg);

//-------------------------------------------------------------------------------------------------

/* clang-format off */
/* STR_ELEM#(v, ...): is the first arg equal any others? */
/* Internal helpers. */
#define _VA_STR_ELEM2(v, a) (strcmp(v, a) == 0)
#define _VA_STR_ELEM3(v, a, b) \
  (_VA_STR_ELEM2(v, a) || (_VA_STR_ELEM2(v, b)))
#define _VA_STR_ELEM4(v, a, b, c) \
  (_VA_STR_ELEM3(v, a, b) || (_VA_STR_ELEM2(v, c)))
#define _VA_STR_ELEM5(v, a, b, c, d) \
  (_VA_STR_ELEM4(v, a, b, c) || (_VA_STR_ELEM2(v, d)))
#define _VA_STR_ELEM6(v, a, b, c, d, e) \
  (_VA_STR_ELEM5(v, a, b, c, d) || (_VA_STR_ELEM2(v, e)))
#define _VA_STR_ELEM7(v, a, b, c, d, e, f) \
  (_VA_STR_ELEM6(v, a, b, c, d, e) || (_VA_STR_ELEM2(v, f)))
#define _VA_STR_ELEM8(v, a, b, c, d, e, f, g) \
  (_VA_STR_ELEM7(v, a, b, c, d, e, f) || (_VA_STR_ELEM2(v, g)))
#define _VA_STR_ELEM9(v, a, b, c, d, e, f, g, h) \
  (_VA_STR_ELEM8(v, a, b, c, d, e, f, g) || (_VA_STR_ELEM2(v, h)))
#define _VA_STR_ELEM10(v, a, b, c, d, e, f, g, h, i) \
  (_VA_STR_ELEM9(v, a, b, c, d, e, f, g, h) || (_VA_STR_ELEM2(v, i)))
#define _VA_STR_ELEM11(v, a, b, c, d, e, f, g, h, i, j) \
  (_VA_STR_ELEM10(v, a, b, c, d, e, f, g, h, i) || (_VA_STR_ELEM2(v, j)))
#define _VA_STR_ELEM12(v, a, b, c, d, e, f, g, h, i, j, k) \
  (_VA_STR_ELEM11(v, a, b, c, d, e, f, g, h, i, j) || (_VA_STR_ELEM2(v, k)))
#define _VA_STR_ELEM13(v, a, b, c, d, e, f, g, h, i, j, k, l) \
  (_VA_STR_ELEM12(v, a, b, c, d, e, f, g, h, i, j, k) || (_VA_STR_ELEM2(v, l)))
#define _VA_STR_ELEM14(v, a, b, c, d, e, f, g, h, i, j, k, l, m) \
  (_VA_STR_ELEM13(v, a, b, c, d, e, f, g, h, i, j, k, l) || (_VA_STR_ELEM2(v, m)))
#define _VA_STR_ELEM15(v, a, b, c, d, e, f, g, h, i, j, k, l, m, n) \
  (_VA_STR_ELEM14(v, a, b, c, d, e, f, g, h, i, j, k, l, m) || (_VA_STR_ELEM2(v, n)))
#define _VA_STR_ELEM16(v, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o) \
  (_VA_STR_ELEM15(v, a, b, c, d, e, f, g, h, i, j, k, l, m, n) || (_VA_STR_ELEM2(v, o)))
#define _VA_STR_ELEM17(v, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p) \
  (_VA_STR_ELEM16(v, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o) || (_VA_STR_ELEM2(v, p)))
/* clang-format on */

/* reusable STR_ELEM macro */
#define STR_ELEM(...) VA_NARGS_CALL_OVERLOAD(_VA_STR_ELEM, __VA_ARGS__)
