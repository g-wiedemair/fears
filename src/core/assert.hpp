#pragma once

#include <cstddef>

/* Utility functions */

void assert_print_pos(const char *file, int line, const char *function, const char *id);
void assert_print_extra(const char *str);
void assert_print_backtrace();
void assert_abort();
void assert_unreachable_print(const char *file, int line, const char *function);

//-------------------------------------------------------------------------------------------------
// Macros

#ifndef NDEBUG

#  ifdef WITH_ASSERT_ABORT
#    define ASSERT_ABORT assert_abort
#  else
#    define ASSERT_ABORT (void)0
#  endif

#  if defined(__GNUC__) || defined(_MSC_VER)
#    define ASSERT_PRINT_POS(a) assert_print_pos(__FILE__, __LINE__, __func__, #a)
#  else
#    define ASSERT_PRINT_POS(a) assert_print_pos(__FILE__, __LINE__, "<?>", #a)
#  endif

#  define fassert(a) \
    (void)((!(a)) ? ((assert_print_backtrace(), ASSERT_PRINT_POS(a), ASSERT_ABORT(), NULL)) : NULL)

#  define fassert_msg(a, msg) \
    (void)((!(a)) ? ((assert_print_backtrace(), \
                      ASSERT_PRINT_POS(a), \
                      assert_print_extra(msg), \
                      ASSERT_ABORT(), \
                      NULL)) : \
                    NULL)
#else
#  define fassert(a) ((void)0)
#  define fassert_msg(a, msg) ((void)0)
#endif

/**
 * Indicates that this line of code should never be executed. If it is reached, it will abort in
 * debug builds and print an error in release builds.
 */
#define unreachable() \
  { \
    assert_unreachable_print(__FILE__, __LINE__, __func__); \
    fassert_msg(0, "This line of code is marked to be unreachable."); \
  } \
  ((void)0)

/**
 * Indicates that this function is not implemented yet.
 */
#define todo() \
  { \
    fassert_msg(0, "Not implemented yet."); \
  } \
  ((void)0)
