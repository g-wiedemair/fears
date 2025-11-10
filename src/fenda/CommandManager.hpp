#pragma once

#include "core/memory.hpp"
#include "fenda/fenda_api.hpp"

class FENDA_API CommandManager {
 private:
  static CommandManager *_instance;

 public:
  static CommandManager *get_handle() {
    static bool bfirst = true;
    if (bfirst) {
      _instance = mem_new<CommandManager>(__func__);
      bfirst = false;
    }
    return _instance;
  }

 private:
  CommandManager();

  template<typename T, typename... Args> friend T *mem_new(const char *, Args &&...);
};
