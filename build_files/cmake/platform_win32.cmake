# -----------------------------------------------------------------------------
# Platform flags

add_definitions(-DWIN32)
if (CMAKE_CL_64)
    add_definitions(-DWIN64)
endif ()

list(APPEND PLATFORM_LINK_LIBS
        advapi32
)


string(APPEND CMAKE_C_FLAGS " /nologo /J /Gd /MP /bigobj /Zc:inline")

set(CMAKE_C_COMPILER_LAUNCHER sccache)
string(APPEND CMAKE_C_FLAGS_DEBUG " /MDd")
string(APPEND CMAKE_C_FLAGS_RELEASE " /MD")

# no default lib
string(APPEND PLATFORM_LINKFLAGS_RELEASE " /NODEFAULTLIB:libcmt.lib /NODEFAULTLIB:libcmtd.lib /NODEFAULTLIB:msvcrt.lib")
string(APPEND PLATFORM_LINKFLAGS_DEBUG " /NODEFAULTLIB:libcmt.lib /NODEFAULTLIB:libcmtd.lib /NODEFAULTLIB:msvcrtd.lib")
