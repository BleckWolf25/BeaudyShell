cmake_minimum_required(VERSION 3.10)
project(BeaudyShell VERSION 0.1.0 LANGUAGES C)

# Set C standard
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Add compiler warnings
if (CMAKE_C_COMPILER_ID MATCHES "GNU|Clang")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Wextra -pedantic")
endif()

# Source files
set(SOURCES
    src/main.c
    src/shell.c
    src/input.c
    src/execute.c
    src/builtins.c
    src/prompt.c
    src/pipes.c
    src/signals.c
    src/utils.c
    src/jobs.c
    src/job_builtins.c
)

# Build executable
add_executable(BeaudyShell ${SOURCES} ${HEADERS})
target_include_directories(BeaudyShell PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)

# Installation
install(TARGETS BeaudyShell DESTINATION bin)

# Documentation
option(BUILD_DOCS "Build documentation" ON)
if(BUILD_DOCS)
    find_package(Doxygen)
    if(DOXYGEN_FOUND)
        set(DOXYGEN_INPUT_DIR ${CMAKE_CURRENT_SOURCE_DIR})
        set(DOXYGEN_OUTPUT_DIR ${CMAKE_CURRENT_BINARY_DIR}/docs)
        configure_file(${CMAKE_CURRENT_SOURCE_DIR}/Doxyfile.in
                      ${CMAKE_CURRENT_BINARY_DIR}/Doxyfile @ONLY)
        add_custom_target(docs
            ${DOXYGEN_EXECUTABLE} ${CMAKE_CURRENT_BINARY_DIR}/Doxyfile
            WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
            COMMENT "Generating API documentation with Doxygen"
            VERBATIM)
    endif()
endif()