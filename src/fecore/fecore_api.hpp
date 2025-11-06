#pragma once

#ifdef WIN32
#ifdef FEAP_DLL
#ifdef fecore_EXPORTS
#define FECORE_API __declspec(dllexport)
#else
#define FECORE_API __declspec(dllimport)
#endif
#else
#define FECORE_API
#endif
#else
#define FECORE_API
#endif
