#pragma once

#ifdef WIN32
#ifdef FEAP_DLL
#ifdef core_EXPORTS
#define CORE_API __declspec(dllexport)
#else
#define CORE_API __declspec(dllimport)
#endif
#else
#define CORE_API
#endif
#else
#define CORE_API
#endif
