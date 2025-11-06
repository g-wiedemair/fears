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
)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# -----------------------------------------------------------------------------

list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/../build_files/cmake/")

# Avoid having an empty `CMAKE_BUILD_TYPE`
if (NOT DEFINED CMAKE_BUILD_TYPE_INIT)
    set(CMAKE_BUILD_TYPE_INIT "Release")
endif ()
mark_as_advanced(CMAKE_BUILD_TYPE_INIT)


# -----------------------------------------------------------------------------
# Redirect Output Files

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin CACHE INTERNAL "" FORCE)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib CACHE INTERNAL "" FORCE)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib CACHE INTERNAL "" FORCE)

# -----------------------------------------------------------------------------
# Set policy

# So library linking is more sane
cmake_policy(SET CMP0003 NEW)
# So syntax problems are errors
cmake_policy(SET CMP0010 NEW)

# -----------------------------------------------------------------------------
# Load macros

include(../build_files/cmake/macros.cmake)

# -----------------------------------------------------------------------------
# Options

# Memory
option(WITH_ASSERT_ABORT "Call abort() when raising an assertion through assert()" ON)
mark_as_advanced(WITH_ASSERT_ABORT)

if (WITH_ASSERT_ABORT)
    add_definitions(-DWITH_ASSERT_ABORT)
endif ()

# -----------------------------------------------------------------------------
# Platform

if (LINUX)
    message(FATAL_ERROR " Not implemented yet.")
elseif (WIN32)
    include(platform_win32)
endif ()

# -----------------------------------------------------------------------------

include_directories(${CMAKE_SOURCE_DIR})

add_subdirectory(core)
add_subdirectory(fecore)
add_subdirectory(feap)
