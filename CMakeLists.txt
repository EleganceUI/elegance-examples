# Every C++ example in one build, sharing one Elegance. Each <name>-cpp folder is also a complete
# project on its own.

cmake_minimum_required(VERSION 4.3)
project(elegance_examples LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Elegance: ELEGANCE_DIR if set, else a checkout beside elegance-examples, else downloaded from
# GitHub.
if (NOT TARGET Elegance::Elegance)
    # The app beside elegance.dll and its dependencies, so Windows finds them.
    set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
    set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
    set(ELEGANCE_DIR "" CACHE PATH "Elegance's source (empty: ../elegance if it's there, else download it)")
    if (NOT ELEGANCE_DIR AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/../elegance/CMakeLists.txt")
        set(ELEGANCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../elegance")
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

add_subdirectory(minimal-cpp)
add_subdirectory(counter-cpp)
add_subdirectory(todos-cpp)
add_subdirectory(layout-cpp)
add_subdirectory(styling-cpp)
add_subdirectory(drawing-cpp)
add_subdirectory(animation-cpp)
add_subdirectory(title-bar-cpp)
add_subdirectory(media-player-cpp)
add_subdirectory(network-cpp)
add_subdirectory(recorder-cpp)
add_subdirectory(desktop-cpp)
add_subdirectory(popover-cpp)
add_subdirectory(navigation-cpp)
add_subdirectory(grids-cpp)
add_subdirectory(effects-cpp)
add_subdirectory(login-cpp)
add_subdirectory(music-cpp)
add_subdirectory(shell-cpp)
add_subdirectory(design-cpp)
add_subdirectory(hello-cpp)
add_subdirectory(preview-cpp)
if (WIN32)
    add_subdirectory(opengl-cpp)
endif ()
