#pragma once

#pragma once

#ifdef WIN32
#  ifdef FEAP_DLL
#    ifdef fenda_EXPORTS
#      define FENDA_API __declspec(dllexport)
#    else
#      define FENDA_API __declspec(dllimport)
#    endif
#  else
#    define FENDA_API
#  endif
#else
#  define FENDA_API
#endif
