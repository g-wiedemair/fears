#include "utils.hpp"

#include <cstring>

#ifdef WIN32
#  include <windows.h>
#endif

int utils::get_app_path(char *pname, size_t pathsize) {
  long result;
  int status = -1;

#ifdef WIN32
  result = GetModuleFileNameA(GetModuleHandleA(NULL), (LPCH)pname, pathsize);
  if (result > 0) {
    // fix slashes
    int len = strlen(pname);
    for (int idx = 0; idx < len; idx++) {
      if (pname[idx] == '\\')
        pname[idx] = '/';
    }
    // File exists, return OK
    status = 0;
  }
#else
  TODO;
#endif

  if (status == 0) {
    char *ch = strrchr(pname, '\\');
    if (ch == 0)
      ch = strrchr(pname, '/');
    if (ch)
      ch[1] = 0;
  }

  return status;
}
