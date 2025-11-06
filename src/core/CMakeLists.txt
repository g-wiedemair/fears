set(SRC
        sys_types.hpp
        assert.hpp
        assert.cpp
        defines_variadic.hpp
        atomic_ops_ext.hpp
        mutex.hpp
        system.hpp
        system.cpp
        string.hpp
        string.cpp
        memory.hpp
        intern/memory_function_pointers.hpp
        intern/memory_alloc_intern.hpp
        intern/memory_alloc.cpp
        intern/memory_usage.cpp
        intern/memory_leak_detector.cpp
        intern/memory_alloc_lockfree.cpp
        intern/memory_alloc_guarded.cpp
        intern/system_win32.cpp
)
if (WIN32)
    list(APPEND SRC
            intern/system_win32.cpp
    )
endif ()

add_library(core ${SRC})
target_link_libraries(core ${PTHREADS_LIBRARIES} ${PLATFORM_LINKLIBS})
setup_platform_linker_flags(core)
