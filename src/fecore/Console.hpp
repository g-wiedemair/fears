#pragma once

#include "core/Vector.hpp"
#include "core/string.hpp"
#include "fecore/LogStream.hpp"
#include "fecore/fecore_api.hpp"

class Console {
 private:
  static Console *_shell;

  bool _active;
  Vector<String, 100> _history;

 public:
  static FECORE_API Console *get_handle();

  FECORE_API ~Console();

  FECORE_API void set_title(const char *title, ...);

  FECORE_API void get_command(int &argc, char **argv);

 private:
  Console();

  void cleanup();

  void write(const char *text, uint16_t att);

  template<typename T, typename... Args> friend T *mem_new(const char *, Args &&...);
};

//-------------------------------------------------------------------------------------------------

class FECORE_API ConsoleStream : public LogStream {
 public:
  void print(const char *sz) override;

  void flush() override;
};
