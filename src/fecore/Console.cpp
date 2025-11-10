#include "Console.hpp"

#include "core/assert.hpp"
#include "core/memory.hpp"

#include <cstdio>

#ifdef WIN32
#  include <shobjidl.h>
#endif

#ifdef WIN32
ITaskbarList3 *task_bar = nullptr;

HANDLE hout = GetStdHandle(STD_OUTPUT_HANDLE);
CONSOLE_SCREEN_BUFFER_INFO csbi;
WORD current_console_attr;
#endif

Console *Console::_shell = nullptr;

Console *Console::get_handle() {
  if (_shell == nullptr) {
    _shell = mem_new<Console>(__func__);
  }
  return _shell;
}

Console::Console() {
  _active = true;

#ifdef WIN32
  if (GetConsoleScreenBufferInfo(hout, &csbi)) {
    current_console_attr = csbi.wAttributes;
  }
#endif
}

Console::~Console() {
  cleanup();
}

void Console::cleanup() {
#ifdef WIN32
  if (task_bar != nullptr) {
    task_bar->Release();
    CoUninitialize();
    task_bar = nullptr;
  }
#endif
}

void Console::set_title(const char *title, ...) {
  if (_active) {
    va_list args;
    char buf[512];
    va_start(args, title);
    vsnprintf(buf, sizeof(buf), title, args);
    va_end(args);

#ifdef WIN32
    SetConsoleTitleA(buf);
#elif defined(LINUX)
    TODO;
#endif
  }
}

void Console::get_command(int &argc, char **argv) {
  static char cmd[512] = {0};
  cmd[0] = 0;

  write("\n>> ", 0x0E);
  fflush(stdin);
  fgets(cmd, 255, stdin);

  // fgets does not remove '\n'
  char *ch = strrchr(cmd, '\n');
  if (ch)
    *ch = 0;

  // check for a percentage sign
  if (cmd[0] == '%') {
    todo();
  }

  // store a copy of the input to the history
  if (strcmp(cmd, "hist") != 0) {
    _history.append(cmd);
  }

  // parse the arguments
  argc = 0;
  int n = 0, b = 0;
  ch = cmd;
  while (*ch) {
    switch (*ch) {
      case ' ':
        if ((b == 0) && (n != 0)) {
          *ch = 0;
          n = 0;
        }
        break;
      case '"':
        if ((b == 0) && (n == 0)) {
          b = 1;
        } else {
          b = 0;
          *ch = 0;
          n = 0;
        }
        break;
      default:
        if (n == 0)
          argv[argc++] = ch;
        n++;
    }
    ch++;
  }
}

void Console::write(const char *text, uint16_t att) {
#ifdef WIN32
  printf("\n");
  SetConsoleTextAttribute(hout, (WORD)att);
  printf("%s", text);
  SetConsoleTextAttribute(hout, current_console_attr);
#else
  printf("%s", text);
#endif
}

void ConsoleStream::print(const char *sz) {
  fprintf(stdout, "%s", sz);
}

void ConsoleStream::flush() {
  fflush(stdout);
}
