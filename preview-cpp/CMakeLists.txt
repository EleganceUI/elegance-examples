# preview: Hot reload and previews: the views live in a module that rebuilds as you edit.
# A complete project: copy this folder to start an app (see the README one folder up).

cmake_minimum_required(VERSION 4.3)
project(preview LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Elegance: ELEGANCE_DIR if set, else a checkout beside elegance-examples, else downloaded from
# GitHub. (elegance-examples' root project adds it once for every example.)
if (NOT TARGET Elegance::Elegance)
    # The app beside elegance.dll and its dependencies, so Windows finds them.
    set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
    set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
    set(ELEGANCE_DIR "" CACHE PATH "Elegance's source (empty: ../../elegance if it's there, else download it)")
    if (NOT ELEGANCE_DIR AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/../../elegance/CMakeLists.txt")
        set(ELEGANCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../elegance")
    endif ()
    if (ELEGANCE_DIR)
        add_subdirectory("${ELEGANCE_DIR}" elegance)
    else ()
        include(FetchContent)
        FetchContent_Declare(elegance
                GIT_REPOSITORY https://github.com/EleganceUI/elegance.git
                GIT_TAG main
                GIT_SHALLOW ON)
        FetchContent_MakeAvailable(elegance)
    endif ()
endif ()

add_executable(preview main.cpp)
target_link_libraries(preview PRIVATE Elegance::Elegance)
elegance_application(preview)

get_target_property(elegance_type Elegance::Elegance TYPE)
if (elegance_type STREQUAL "SHARED_LIBRARY")
    elegance_views_module(preview_views views/root.cpp views/profile_card.cpp)
    elegance_hot_reload(preview MODULE preview_views WATCH views)
    target_compile_definitions(preview PRIVATE ELG_PREVIEW_HOT_RELOAD=1)
    set(preview_targets preview_views preview)
else ()
    # Static release builds keep one Elegance runtime and do not load a views module.
    target_sources(preview PRIVATE views/root.cpp views/profile_card.cpp)
    set(preview_targets preview)
endif ()

foreach (target IN LISTS preview_targets)
    if (MSVC)
        target_compile_options(${target} PRIVATE /W4 /permissive- /utf-8)
    else ()
        target_compile_options(${target} PRIVATE -Wall -Wextra -Wpedantic)
    endif ()
endforeach ()
