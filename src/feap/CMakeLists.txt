set(SRC
        feap.cpp
        FeapApp.cpp
        FeapApp.hpp
)

add_executable(feap ${SRC})
target_link_libraries(feap fecore)
