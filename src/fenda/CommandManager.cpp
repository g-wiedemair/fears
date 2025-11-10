#include "CommandManager.hpp"

CommandManager *CommandManager::instance_ = nullptr;

CommandManager::CommandManager() {
  this->register_command(mem_new<CmdHelp>(__func__), "help", "print available commands");
  this->register_command(mem_new<CmdVersion>(__func__), "version", "print version information");
}

CommandManager::~CommandManager() {
  Vector<Command *>::iterator it;
  for (it = cmds_.begin(); it != cmds_.end(); ++it) {
    mem_delete(*it);
  }
}

void CommandManager::register_command(Command *cmd, const char *name, const char *desc) {
  cmd->set_name(name);
  cmd->set_description(desc);
  add_command(cmd);
}
