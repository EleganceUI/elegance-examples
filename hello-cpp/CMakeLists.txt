cmake_minimum_required(VERSION 3.28)
project(HelloElegence LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# The app and elegence.dll (with its dependencies) share one folder, so Windows finds the DLLs.
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Elegence is built as part of this project (it has no installed package yet).
set(ELEGENCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../ElegenceMain" CACHE PATH "The Elegence source tree")
add_subdirectory(${ELEGENCE_DIR} elegence)

add_executable(hello-elegence
        src/main.cpp
        src/root.cpp
        src/components/hello_world.cpp
)
target_include_directories(hello-elegence PRIVATE src)
target_link_libraries(hello-elegence PRIVATE Elegence::Elegence)

if (MSVC)
    target_compile_options(hello-elegence PRIVATE /W4 /permissive- /utf-8)
else ()
    target_compile_options(hello-elegence PRIVATE -Wall -Wextra -Wpedantic)
endif ()
