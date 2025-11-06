function(get_feap_version)

    # set upper case <PROJECT>_VERSION_... variables
    string(TOUPPER ${PROJECT_NAME} UPPER_PROJECT_NAME)
    set(${UPPER_PROJECT_NAME}_VERSION ${PROJECT_VERSION} CACHE INTERNAL "" FORCE)
    set(${UPPER_PROJECT_NAME}_VERSION_MAJOR ${PROJECT_VERSION_MAJOR})
    set(${UPPER_PROJECT_NAME}_VERSION_MINOR ${PROJECT_VERSION_MINOR})
    set(${UPPER_PROJECT_NAME}_VERSION_PATCH ${PROJECT_VERSION_PATCH})

    # Version file
    configure_file("${CMAKE_SOURCE_DIR}/../build_files/feap_version.hpp.in" "${CMAKE_BINARY_DIR}/src/core/feap_version.hpp")

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

macro(windows_generate_manifest)
    set(options)
    set(oneValueArgs OUTPUT NAME)
    set(multiValueArgs FILES)
    cmake_parse_arguments(
            WINDOWS_MANIFEST
            " ${options}"
            "${oneValueArgs}"
            "${multiValueArgs}"
            ${ARGN}
    )
    set(MANIFEST_LIBS "")
    foreach (lib ${WINDOWS_MANIFEST_FILES})
        get_filename_component(filename ${lib} NAME)
        set(MANIFEST_LIBS "${MANIFEST_LIBS} <file name=\"${filename}\"/>\n")
    endforeach ()
    configure_file(
            ${CMAKE_SOURCE_DIR}/../release/windows/manifest/feap.manifest.in
            ${WINDOWS_MANIFEST_OUTPUT}
            @ONLY
    )
endmacro()

macro(windows_generate_shared_manifest)
    if (WINDOWS_SHARED_MANIFEST_DEBUG)
        windows_generate_manifest(
                FILES "${WINDOWS_SHARED_MANIFEST_DEBUG}"
                OUTPUT "${CMAKE_BINARY_DIR}/Debug/feap.shared.manifest"
                NAME "feap.shared"
        )
        install(
                FILES ${CMAKE_BINARY_DIR}/Debug/feap.shared.manifest
                DESTINATION "feap.shared"
                CONFIGURATIONS Debug
        )
    endif ()
    if (WINDOWS_SHARED_MANIFEST_RELEASE)
        windows_generate_manifest(
                FILES "${WINDOWS_SHARED_MANIFEST_RELEASE}"
                OUTPUT "${CMAKE_BINARY_DIR}/Release/feap.shared.manifest"
                NAME "feap.shared"
        )
        install(
                FILES ${CMAKE_BINARY_DIR}/Release/feap.shared.manifest
                DESTINATION "feap.shared"
                CONFIGURATIONS Release;RelWithDebInfo;MinSizeRel
        )
    endif ()
endmacro()

macro(windows_install_shared_manifest)
    set(options OPTIONAL DEBUG RELEASE ALL)
    set(oneValueArgs)
    set(multiValueArgs FILES)
    cmake_parse_arguments(
            WINDOWS_INSTALL
            "${options}"
            "${oneValueArgs}"
            "${multiValueArgs}"
            ${ARGN}
    )
    # If none of the options are set assume ALL
    unset(WINDOWS_CONFIGURATIONS)
    if (NOT WINDOWS_INSTALL_ALL AND
            NOT WINDOWS_INSTALL_DEBUG AND
            NOT WINDOWS_INSTALL_RELEASE)
        set(WINDOWS_INSTALL_ALL TRUE)
    endif ()
    # If all is et, turn both DEBUG and RELEASE on
    if (WINDOWS_INSTALL_ALL)
        set(WINDOWS_INSTALL_DEBUG TRUE)
        set(WINDOWS_INSTALL_RELEASE TRUE)
    endif ()
    if (WINDOWS_INSTALL_DEBUG)
        set(WINDOWS_CONFIGURATIONS "${WINDOWS_CONFIGURATIONS};Debug")
    endif ()
    if (WINDOWS_INSTALL_RELEASE)
        set(WINDOWS_CONFIGURATIONS "${WINDOWS_CONFIGURATIONS};Release;RelWithDebInfo;MinSizeRel")
    endif ()

    # Feap executable with manifest
    if (WINDOWS_INSTALL_DEBUG)
        list(APPEND WINDOWS_SHARED_MANIFEST_DEBUG ${WINDOWS_INSTALL_FILES})
    endif ()
    if (WINDOWS_INSTALL_RELEASE)
        list(APPEND WINDOWS_SHARED_MANIFEST_RELEASE ${WINDOWS_INSTALL_FILES})
    endif ()
    install(
            FILES ${WINDOWS_INSTALL_FILES}
            DESTINATION "feap.shared"
            CONFIGURATIONS ${WINDOWS_CONFIGURATIONS}
    )
endmacro()
