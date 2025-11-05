# -----------------------------------------------------------------------------
# Platform flags

add_definitions(-DWIN32)
if (CMAKE_CL_64)
    add_definitions(-DWIN64)
endif ()

string(APPEND CMAKE_Fortran_FLAGS_DEBUG " /Z7 /debug:full")

set(CMAKE_C_COMPILER_LAUNCHER sccache)
string(APPEND CMAKE_C_FLAGS " /nologo /J /Gd /MP /bigobj /Zc:inline")
string(APPEND CMAKE_C_FLAGS_DEBUG " /MDd /Wall")
string(APPEND CMAKE_C_FLAGS_RELEASE " /MD /W1")

# no default lib
string(APPEND PLATFORM_LINKFLAGS_RELEASE " /NODEFAULTLIB:libcmt.lib /NODEFAULTLIB:libcmtd.lib /NODEFAULTLIB:msvcrtd.lib")
string(APPEND PLATFORM_LINKFLAGS_DEBUG " /NODEFAULTLIB:libcmt.lib /NODEFAULTLIB:libcmtd.lib /NODEFAULTLIB:msvcrt.lib")
