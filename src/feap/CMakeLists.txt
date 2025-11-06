set(SRC
        feap.cpp
        FeapApp.cpp
        FeapApp.hpp
)

add_executable(feap ${SRC})
target_link_libraries(feap core fecore fenda)
setup_platform_linker_flags(feap)
