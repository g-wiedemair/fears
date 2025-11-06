function(get_feap_version)

    # set upper case <PROJECT>_VERSION_... variables
    string(TOUPPER ${PROJECT_NAME} UPPER_PROJECT_NAME)
    set(${UPPER_PROJECT_NAME}_VERSION ${PROJECT_VERSION} CACHE INTERNAL "" FORCE)
    set(${UPPER_PROJECT_NAME}_VERSION_MAJOR ${PROJECT_VERSION_MAJOR})
    set(${UPPER_PROJECT_NAME}_VERSION_MINOR ${PROJECT_VERSION_MINOR})
    set(${UPPER_PROJECT_NAME}_VERSION_PATCH ${PROJECT_VERSION_PATCH})

    # Version file
    configure_file("${CMAKE_SOURCE_DIR}/../build_files/feap_version.hpp.in" "${CMAKE_BINARY_DIR}/fecore/feap_version.hpp")

endfunction()

# -----------------------------------------------------------------------------

function(setup_platform_linker_flags
        target
)
    set_property(
            TARGET ${target} APPEND_STRING PROPERTY
            LINK_FLAGS " ${PLATFORM_LINKFLAGS}"
    )
    set_property(
            TARGET ${target} APPEND_STRING PROPERTY
            LINK_FLAGS_RELEASE " ${PLATFORM_LINKFLAGS_RELEASE}"
    )
    set_property(
            TARGET ${target} APPEND_STRING PROPERTY
            LINK_FLAGS_DEBUG " ${PLATFORM_LINKFLAGS_DEBUG}"
    )

    get_target_property(target_type ${target} TYPE)
    if (target_type STREQUAL "EXECUTABLE")
        set_property(
                TARGET ${target} APPEND_STRING PROPERTY
                LINK_FLAGS " ${PLATFORM_LINKFLAGS_EXECUTABLE}"
        )
    endif ()
endfunction()

#-----------------------------------------------------------------------------------------------------------------------
