# login: A modern sign-in screen with a QR code.
# A complete project: copy this folder to start an app (see the README one folder up).

cmake_minimum_required(VERSION 4.3)
project(login LANGUAGES CXX)

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

find_package(unofficial-nayuki-qr-code-generator CONFIG REQUIRED)

add_executable(login
        main.cpp
        qr_code.cpp
)
target_link_libraries(login PRIVATE
        Elegance::Elegance
        unofficial::nayuki-qr-code-generator::nayuki-qr-code-generator
)

if (MSVC)
    target_compile_options(login PRIVATE /W4 /permissive- /utf-8)
else ()
    target_compile_options(login PRIVATE -Wall -Wextra -Wpedantic)
endif ()
