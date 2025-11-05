set(SRC
        foo.f90
        bar.f
)

add_library(foo ${SRC})
target_link_libraries(foo)
setup_platform_linker_flags(foo)

install(TARGETS foo DESTINATION .)
