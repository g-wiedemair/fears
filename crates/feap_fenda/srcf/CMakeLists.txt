cmake_minimum_required(VERSION 3.29)

project(foo
    VERSION 0.1.0
    LANGUAGES Fortran
)

set(SRC
        foo.f90
        bar.f
)

add_library(foo ${SRC})
target_link_libraries(foo)
setup_platform_linker_flags(foo)

install(TARGETS foo DESTINATION .)
