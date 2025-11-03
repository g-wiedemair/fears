cmake_minimum_required(VERSION 3.29)

# NOTE: We don't allow in-source builds.
if (${CMAKE_SOURCE_DIR} STREQUAL ${CMAKE_BINARY_DIR})
    message(FATAL_ERROR
            "CMake generation is not allowed within the source directory!"
            "\n Remove \"${CMAKE_SOURCE_DIR}/CMakeCache.txt\""
            "\n then try again from another directory."
    )
endif ()

# -----------------------------------------------------------------------------
# Initialize project

project(fenda
        VERSION 0.1.0
        LANGUAGES Fortran C
)

# -----------------------------------------------------------------------------

list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/../../build_files/cmake/")
include(../../build_files/cmake/macros.cmake)


# Avoid having an empty `CMAKE_BUILD_TYPE`
if (NOT DEFINED CMAKE_BUILD_TYPE_INIT)
    set(CMAKE_BUILD_TYPE_INIT "Release")
endif ()
mark_as_advanced(CMAKE_BUILD_TYPE_INIT)

# -----------------------------------------------------------------------------
# Redirect Output Files

set(EXECUTABLE_OUTPUT_PATH ${CMAKE_BINARY_DIR}/bin CACHE INTERNAL "" FORCE)
set(LIBRARY_OUTPUT_PATH ${CMAKE_BINARY_DIR}/lib CACHE INTERNAL "" FORCE)

# -----------------------------------------------------------------------------
# Options

option(WITH_MKL_LIBRARY "Enable pardiso solver" ON)
if (WITH_MKL_LIBRARY)
    # MKL library
    set(MKL_INTERFACE lp64) # Use the 64-bit integer interface

    find_package(MKL)
    if (MKL_not_found)
        message(WARNING "oneMKL not found. Pardiso Solver disabled.")
        set(WITH_MKL_LIBRARY OFF)
    endif ()
endif ()

# -----------------------------------------------------------------------------
# Platform

if (LINUX)
    message(FATAL_ERROR " Not implemented yet.")
elseif (WIN32)
    include(platform_win32)
endif ()

# -----------------------------------------------------------------------------

# include_directories(include)

# add_subdirectory(brun)
# add_subdirectory(Xdb)
# add_subdirectory(i3p)
# add_subdirectory(unv)
# add_subdirectory(usl)
# add_subdirectory(fdafg)
# add_subdirectory(toka_lib)
# add_subdirectory(fenda_lib)
# add_subdirectory(fenda)

add_subdirectory(srcf)
