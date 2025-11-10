#pragma once

#include "core/assert.hpp"
#include "core/memory.hpp"
#include "fenda/fenda_api.hpp"

class FENDA_API Command {
 private:
  char *name_;
  char *desc_;

 public:
  Command() {
    name_ = nullptr;
    desc_ = nullptr;
  }

  virtual ~Command() {
    mem_free(name_);
    mem_free(desc_);
  }

  void set_name(const char *name) {
    size_t l = strlen(name);
    fassert(l);
    name_ = (char *)mem_malloc_array(l + 1, sizeof(char), __func__);
    strcpy(name_, name);
  }

  void set_description(const char *desc) {
    size_t l = strlen(desc);
    fassert(l);
    desc_ = (char *)mem_malloc_array(l + 1, sizeof(char), __func__);
    strcpy(desc_, desc);
  }

  const char *get_name() {
    return name_;
  }

  const char *get_description() {
    return desc_;
  }

  virtual int run(int argc, char **argv) = 0;

 private:
  friend class CommandManager;
};

//-------------------------------------------------------------------------------------------------

class CmdHelp : public Command {
 public:
  int run(int argc, char **argv);
};

class CmdVersion : public Command {
 public:
  int run(int argc, char **argv);
};
