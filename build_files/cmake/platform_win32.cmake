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
        -DNOMINMAX
)

# Needed, otherwise system encoding causes utf-8 encoding to fail in some cases (C4819)
add_compile_options("$<$<C_COMPILER_ID:MSVC>:/utf-8>")
add_compile_options("$<$<CXX_COMPILER_ID:MSVC>:/utf-8>")

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

#-----------------------------------------------------------------------------------------------------------------------
# Libraries

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

if (WITH_TBB)
    if (EXISTS ${LIBDIR}/tbb/lib/tbb12.lib) # 4.4
        set(TBB_LIBRARIES
                optimized ${LIBDIR}/tbb/lib/tbb12.lib
                debug ${LIBDIR}/tbb/lib/tbb12_debug.lib
        )
        set(TBB_INCLUDE_DIRS ${LIBDIR}/tbb/include)
        if (WITH_TBB_MALLOC_PROXY)
            set(TBB_MALLOC_LIBRARIES
                    optimized ${LIBDIR}/tbb/lib/tbbmalloc.lib
                    debug ${LIBDIR}/tbb/lib/tbbmalloc_debug.lib
            )
            add_definitions(-DWITH_TBB_MALLOC)
        endif ()
    else ()
        message(WARNING "TBB library not found. Setting WITH_TBB to OFF")
        set(WITH_TBB OFF)
        set(WITH_TBB_MALLOC_PROXY OFF)
    endif ()
endif ()

list(APPEND PLATFORM_LINKLIBS
        version Dbghelp Shlwapi
)
