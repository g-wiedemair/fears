set(SRC
        fecore_api.hpp
        FeKernel.hpp
        FeKernel.cpp
)

add_library(fecore SHARED ${SRC})
target_link_libraries(fecore core)
setup_platform_linker_flags(fecore)
