set(SRC
        fenda_api.hpp
        fenda.hpp
        fenda.cpp
        hello.cpp
        config.cpp
        FeapConfig.cpp
        FeapConfig.hpp
)

add_library(fenda SHARED ${SRC})
target_link_libraries(fenda core fecore)
