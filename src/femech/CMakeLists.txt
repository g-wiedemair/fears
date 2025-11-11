set(SRC
        FeMechModel.hpp
        FeMechModel.cpp
)

add_library(femech SHARED ${SRC})
target_link_libraries(femech core)
