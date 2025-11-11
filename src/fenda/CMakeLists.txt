set(SRC
        fenda_api.hpp
        fenda.hpp
        fenda.cpp
        hello.cpp
        config.cpp
        FeapConfig.cpp
        FeapConfig.hpp
        FeapModel.cpp
        FeapModel.hpp
        LogFile.cpp
        LogFile.hpp
)

add_library(fenda SHARED ${SRC})
target_link_libraries(fenda core fecore)
