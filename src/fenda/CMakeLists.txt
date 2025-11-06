set(SRC
        fenda_api.hpp
        fenda.hpp
        hello.cpp
)

add_library(fenda SHARED ${SRC})
target_link_libraries(fenda core fecore)
