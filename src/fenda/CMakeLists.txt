set(SRC
        fenda_api.hpp
        fenda.hpp
        fenda.cpp
        hello.cpp
        config.cpp
        FeapConfig.cpp
        FeapConfig.hpp
        Interruption.cpp
        Interruption.hpp
        CommandManager.cpp
        CommandManager.hpp
        Command.hpp
        Command.cpp
)

add_library(fenda SHARED ${SRC})
target_link_libraries(fenda core fecore)
