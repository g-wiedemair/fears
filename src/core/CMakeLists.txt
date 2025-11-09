set(SRC
        sys_types.hpp
        assert.hpp
        assert.cpp
        defines_variadic.hpp
        atomic_ops_ext.hpp
        utildefines.hpp
        mutex.hpp
        system.hpp
        system.cpp
        memory.hpp
        allocator.hpp
        string.hpp
        string.cpp
        array.hpp
        map.hpp
        intern/memory_inline.hpp
        intern/memory_function_pointers.hpp
        intern/memory_alloc_intern.hpp
        intern/memory_alloc.cpp
        intern/memory_usage.cpp
        intern/memory_leak_detector.cpp
        intern/memory_alloc_lockfree.cpp
        intern/memory_alloc_guarded.cpp
        intern/system_win32.cpp
        intern/memory_utils.hpp
        intern/probing_strategy.hpp
        intern/hash.hpp
        intern/hash_tables.hpp
        intern/map_slots.hpp
)
if (WIN32)
    list(APPEND SRC
            intern/system_win32.cpp
    )
endif ()

add_library(core SHARED ${SRC})
target_link_libraries(core ${PTHREADS_LIBRARIES} ${TBB_LIBRARIES} ${PLATFORM_LINKLIBS})
include_directories(SYSTEM ${TBB_INCLUDE_DIRS})
setup_platform_linker_flags(core)

if (WITH_TBB)
    add_custom_command(TARGET core POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<$<CONFIG:Debug>:"${LIBDIR}/tbb/bin/tbb12_debug.dll">
            $<$<CONFIG:Release>:"${LIBDIR}/tbb/bin/tbb12.dll">
            "$<TARGET_FILE_DIR:core>"
    )
endif ()
