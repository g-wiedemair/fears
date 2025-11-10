set(SRC
        feap.cpp
        FeapApp.cpp
        FeapApp.hpp
)

if (WIN32)
    list(APPEND SRC
            ${CMAKE_BINARY_DIR}/feap.exe.manifest
    )
endif ()

add_executable(feap ${SRC})
target_link_libraries(feap core fecore fenda)
setup_platform_linker_flags(feap)
