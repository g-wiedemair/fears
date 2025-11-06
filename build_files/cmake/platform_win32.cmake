# -----------------------------------------------------------------------------
# Platform flags

add_definitions(-DWIN32 -DFEAP_DLL)
if (CMAKE_CL_64)
    add_definitions(-DWIN64)
endif ()

add_definitions(
        -D_CRT_NONSTDC_NO_DEPRECATE
        -D_CRT_SECURE_NO_DEPRECATE
        -D_SCL_SECURE_NO_DEPRECATE
        -D_CONSOLE
        -D_LIB
)

string(APPEND CMAKE_Fortran_FLAGS_DEBUG " /Z7 /debug:full")

set(CMAKE_C_COMPILER_LAUNCHER sccache)
string(APPEND CMAKE_C_FLAGS " /nologo /J /Gd /MP /bigobj /Zc:inline")
string(APPEND CMAKE_C_FLAGS_DEBUG " /MDd Z7")
string(APPEND CMAKE_C_FLAGS_RELEASE " /MD Z7")

set(CMAKE_CXX_COMPILER_LAUNCHER sccache)
string(APPEND CMAKE_CXX_FLAGS " /nologo /J /Gd /MP /bigobj /Zc:inline")
string(APPEND CMAKE_CXX_FLAGS_DEBUG " /MDd /Z7")
string(APPEND CMAKE_CXX_FLAGS_RELEASE " /MD /Z7")

string(APPEND PLATFORM_LINKFLAGS " /SUBSYSTEM:CONSOLE /STACK:2097152")

# no default lib
string(APPEND PLATFORM_LINKFLAGS_RELEASE " ${PLATFORM_LINKFLAGS} /NODEFAULTLIB:libcmt.lib /NODEFAULTLIB:libcmtd.lib /NODEFAULTLIB:msvcrtd.lib")
string(APPEND PLATFORM_LINKFLAGS_DEBUG " ${PLATFORM_LINKFLAGS} /NODEFAULTLIB:libcmt.lib /NODEFAULTLIB:libcmtd.lib /NODEFAULTLIB:msvcrt.lib")


if (CMAKE_C_COMPILER_ID STREQUAL "MSVC")
    set(_WARNINGS
            "/W3"
            # disable:
            "/wd4018"  # signed/unsigned mismatch
            "/wd4200"  # zero-sized array in struct/union
            "/wd4244"  # conversion from 'type1' to 'type2'
            "/wd4267"  # conversion from 'size_t' to 'type', possible loss of data
            "/wd4353"  # constant 0 as function expression
            "/wd4848"  # 'no_unique_address' is a vendor extension in C++17
            # errors:
            "/we4431"  # missing type specifier - int assumed
    )

    string(REPLACE ";" " " _WARNINGS "${_WARNINGS}")
    set(C_WARNINGS "${_WARNINGS}")
    set(CXX_WARNINGS "${_WARNINGS}")
    unset(_WARNINGS)
endif ()

# Include warnings first, so its possible to disable them with user defined flags
# eg: -Wno-uninitialized
set(CMAKE_C_FLAGS "${C_WARNINGS} ${CMAKE_C_FLAGS} ${PLATFORM_CFLAGS}")
set(CMAKE_CXX_FLAGS "${CXX_WARNINGS} ${CMAKE_CXX_FLAGS} ${PLATFORM_CXXFLAGS}")

#-----------------------------------------------------------------------------------------------------------------------
if (NOT DEFINED LIBDIR)
    if (CMAKE_CL_64)
        if (CMAKE_SYSTEMPROCESSOR STREQUAL "ARM64")
            set(LIBDIR_BASE "lib-windows_arm64")
        else ()
            set(LIBDIR_BASE "lib-windows_x64")
        endif ()
    else ()
        message(FATAL_ERROR
                "32 bit compiler detected, "
                "feap no longer provides pre-build libraries for 32 bit windows, "
                "please set the LIBDIR cmake variable to your own library folder"
        )
    endif ()
    set(LIBDIR ${CMAKE_SOURCE_DIR}/../lib/${LIBDIR_BASE})

    if (NOT EXISTS "${LIBDIR}/.git")
        message(FATAL_ERROR
                "\n\nWindows requires pre-compiled libs at: '${LIBDIR}'. "
                "Please fetch the appropriate git repo."
        )
    endif ()
endif ()

set(PTHREADS_INCLUDE_DIRS ${LIBDIR}/pthreads/include)
set(PTHREADS_LIBRARIES ${LIBDIR}/pthreads/lib/pthreadVC3.lib)
# used in many places so include globally, like OpenGL
include_directories(SYSTEM "${PTHREADS_INCLUDE_DIRS}")

list(APPEND PLATFORM_LINKLIBS
        ws2_32 version Dbghelp Shlwapi
)
