cmake_minimum_required(VERSION 3.29)

# NOTE: We don't allow in-source builds.
if (${CMAKE_SOURCE_DIR} STREQUAL ${CMAKE_BINARY_DIR})
    message(FATAL_ERROR
            "CMake generation is not allowed within the source directory!"
            "\n Remove \"${CMAKE_SOURCE_DIR}/CMakeCache.txt\""
            "\n then try again from another directory."
    )
endif ()

# ----------------------------------------------------------------------------------------------------------------------
# Initialize project

project(Feap
        VERSION 0.1.0
)
set(FEAP_VERSION_CYCLE "alpha")
set(FEAP_VERSION_SUFFIX "")

# ----------------------------------------------------------------------------------------------------------------------

list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/../build_files/cmake/")

# Avoid having an empty `CMAKE_BUILD_TYPE`
if (NOT DEFINED CMAKE_BUILD_TYPE_INIT)
    set(CMAKE_BUILD_TYPE_INIT "Release")
endif ()
mark_as_advanced(CMAKE_BUILD_TYPE_INIT)


# ----------------------------------------------------------------------------------------------------------------------
# Redirect Output Files

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin CACHE INTERNAL "" FORCE)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib CACHE INTERNAL "" FORCE)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib CACHE INTERNAL "" FORCE)

# ----------------------------------------------------------------------------------------------------------------------
# Set policy

# So library linking is more sane
cmake_policy(SET CMP0003 NEW)
# So syntax problems are errors
cmake_policy(SET CMP0010 NEW)

# ----------------------------------------------------------------------------------------------------------------------
# Load macros

include(../build_files/cmake/macros.cmake)

# -----------------------------------------------------------------------------
# Test Compiler Support

if (CMAKE_COMPILER_IS_GNUCC)
    if ("${CMAKE_C_COMPILER_VERSION}" VERSION_LESS "11.0.0")
        message(FATAL_ERROR
                "The minimum supported version of GCC is 11.0.0, found C compiler: "
                "${CMAKE_C_COMPILER_VERSION}"
        )
    endif ()
    if ("${CMAKE_CXX_COMPILER_VERSION}" VERSION_LESS "11.0.0")
        message(FATAL_ERROR
                "The minimum supported version of GCC is 11.0.0, found C++ compiler: "
                "${CMAKE_CXX_COMPILER_VERSION}"
        )
    endif ()

elseif (CMAKE_C_COMPILER_ID MATCHES "Clang")

elseif (CMAKE_CXX_COMPILER_ID STREQUAL "MSVC")
    if (MSVC_VERSION VERSION_LESS "1928")
        message(FATAL_ERROR "The minimum supported version of MSVC is 2019 (16.9.16)")
    endif ()
endif ()

#-----------------------------------------------------------------------------------------------------------------------
# Compiler settings

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
# Do not enable compiler specific language extensions.
set(CMAKE_CXX_EXTENSIONS OFF)

if (CMAKE_COMPILER_IS_GNUCC)
    message(FATAL_ERROR "NIY")

elseif (CMAKE_C_COMPILER_ID MATCHES "Clang")
    message(FATAL_ERROR "NIY")

elseif (CMAKE_C_COMPILER_ID STREQUAL "Intel")
    message(FATAL_ERROR "NIY")

elseif (CMAKE_C_COMPILER_ID STREQUAL "MSVC")
    set(_WARNINGS
            "/W3"
            # disable:
            "/wd4018"  # signed/unsigned mismatch
            "/wd4200"  # zero-sized array in struct/union
            "/wd4244"  # conversion from 'type1' to 'type2'
            "/wd4267"  # conversion from 'size_t' to 'type', possible loss of data
            "/wd4848"  # 'no_unique_address' is a vendor extension in C++17
            # errors:
            "/we4431"  # missing type specifier - int assumed
    )

    string(REPLACE ";" " " _WARNINGS "${_WARNINGS}")
    set(C_WARNINGS "${_WARNINGS}")
    set(CXX_WARNINGS "${_WARNINGS}")
    unset(_WARNINGS)
endif ()

# XCode enables additional waring flags by default.
if (XCODE)
    set(CMAKE_XCODE_ATTRIBUTE_GCC_WARN_64_TO_32_BIT_CONVERSION NO)
endif ()

set(CMAKE_C_FLAGS "${C_WARNINGS} ${CMAKE_C_FLAGS} ${PLATFORM_CFLAGS}")
set(CMAKE_CXX_FLAGS "${CXX_WARNINGS} ${CMAKE_CXX_FLAGS} ${PLATFORM_CFLAGS}")

# -----------------------------------------------------------------------------
# Set Default Configuration Options

get_feap_version()

if (${CMAKE_VERSION} VERSION_GREATER_EQUAL "3.19")
    set(CMAKE_OPTIMIZE_DEPENDENCIES ON CACHE INTERNAL "")
endif ()

# ----------------------------------------------------------------------------------------------------------------------
# Options

option(WITH_ASSERT_ABORT "Call abort() when raising an assertion through assert()" ON)
mark_as_advanced(WITH_ASSERT_ABORT)

option(WITH_TBB "Enable multi-threading." ON)

# TBB malloc is only supported on for windows currently
if (WIN32 AND NOT CMAKE_SYSTEM_PROCESSOR STREQUAL "ARM64")
    option(WITH_TBB_MALLOC_PROXY "Enable the TBB malloc replacement" ON)
endif ()

option(WITH_LOG_PTHREADS "Enable multi-threaded logging" ON)

#-----------------------------------------------------------------------------------------------------------------------
# Option definitions

if (WITH_ASSERT_ABORT)
    add_definitions(-DWITH_ASSERT_ABORT)
endif ()

if (WITH_TBB)
    add_definitions(-DWITH_TBB)
endif ()

if (WITH_LOG_PTHREADS)
    add_definitions(-DWITH_LOG_PTHREADS)
endif ()

#-----------------------------------------------------------------------------------------------------------------------
# Prefix

set(BUILD_TYPE {CMAKE_BUILD_TYPE})

# By default we want to install to the directory we are compiling our executables
# unless specified otherwise, which we currently do not allow
if (CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
    if (WIN32)
        set(CMAKE_INSTALL_PREFIX ${EXECUTABLE_OUTPUT_PATH}/\${BUILD_TYPE} CACHE PATH "default install path" FORCE)
    elseif (APPLE)
        set(CMAKE_INSTALL_PREFIX ${EXECUTABLE_OUTPUT_PATH}/\${BUILD_TYPE} CACHE PATH "default install path" FORCE)
    endif ()
endif ()

# Effective install path including config directory, as a generator expression.
get_property(GENERATOR_IS_MULTI_CONFIG GLOBAL PROPERTY GENERATOR_IS_MULTI_CONFIG)
if (GENERATOR_IS_MULTI_CONFIG)
    string(
            REPLACE "\${BUILD_TYPE}" "$<CONFIG>"
            CMAKE_INSTALL_PREFIX_WITH_CONFIG ${CMAKE_INSTALL_PREFIX}
    )
else ()
    string(
            REPLACE "\${BUILD_TYPE}" ""
            CMAKE_INSTALL_PREFIX_WITH_CONFIG ${CMAKE_INSTALL_PREFIX}
    )
endif ()

# ----------------------------------------------------------------------------------------------------------------------
# Platform

if (LINUX)
    message(FATAL_ERROR " Not implemented yet.")
elseif (WIN32)
    include(platform_win32)
endif ()

# ----------------------------------------------------------------------------------------------------------------------

include_directories(${CMAKE_SOURCE_DIR})
include_directories(${CMAKE_BINARY_DIR})

add_subdirectory(core)
add_subdirectory(fecore)
add_subdirectory(femech)
add_subdirectory(fenda)
add_subdirectory(feap)
