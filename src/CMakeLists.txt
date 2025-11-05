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

set(EXECUTABLE_OUTPUT_PATH ${CMAKE_BINARY_DIR}/bin CACHE INTERNAL "" FORCE)
set(LIBRARY_OUTPUT_PATH ${CMAKE_BINARY_DIR}/lib CACHE INTERNAL "" FORCE)

# -----------------------------------------------------------------------------
# Options


# -----------------------------------------------------------------------------
# Platform

if (LINUX)
    message(FATAL_ERROR " Not implemented yet.")
elseif (WIN32)
    include(platform_win32)
endif ()

# -----------------------------------------------------------------------------

include_directories(${CMAKE_SOURCE_DIR})

#add_subdirectory(core)
add_subdirectory(feap)
