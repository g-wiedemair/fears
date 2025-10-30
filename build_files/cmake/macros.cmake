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
