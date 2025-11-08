set(SRC
        fecore_api.hpp
        version.cpp
        utils.cpp
        utils.hpp
        CmdOptions.cpp
        CmdOptions.hpp
        log.cpp
        log.hpp
        LogStream.cpp
        LogStream.hpp
        Console.cpp
        Console.hpp
        FeKernel.hpp
        FeKernel.cpp
)

add_library(fecore SHARED ${SRC})
target_link_libraries(fecore core)
setup_platform_linker_flags(fecore)
