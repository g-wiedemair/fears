#pragma once

#include "core/StringRef.hpp"
#include "core/VectorMap.hpp"
#include "core/memory.hpp"
#include "feap/Command.hpp"

#define COMMAND_COUNT 10

class CommandManager {
 private:
  static CommandManager *instance_;

  VectorMap<StringRef, Command *, COMMAND_COUNT> cmds_;

 public:
  static inline CommandManager *get_handle() {
    static bool bfirst = true;
    if (bfirst) {
      instance_ = mem_new<CommandManager>(__func__);
      bfirst = false;
    }
    return instance_;
  }

  ~CommandManager();

 public:
  inline int64_t size() {
    return cmds_.size();
  }

  inline void add_command(Command *cmd) {
    cmds_.add(cmd->name_, cmd);
  }

  inline Command *find(const char *cmd) {
    Vector<Command *>::iterator it;
    for (it = cmds_.begin(); it != cmds_.end(); ++it) {
      if (strcmp(cmd, (*it)->get_name()) == 0)
        return *it;
    }

    return nullptr;
  }

  typedef Vector<Command *>::iterator CmdIterator;
  CmdIterator begin() {
    return cmds_.begin();
  }
  CmdIterator end() {
    return cmds_.end();
  }

 private:
  CommandManager();

  void register_command(Command *cmd, const char *name, const char *desc);

  template<typename T, typename... Args> friend T *mem_new(const char *, Args &&...);
};
