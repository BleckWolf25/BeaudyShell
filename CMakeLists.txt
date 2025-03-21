cmake_minimum_required(VERSION 3.10)
project(BeaudyShell 
        VERSION 0.1.0 
        DESCRIPTION "A fast, interactive Unix shell"
        HOMEPAGE_URL "https://github.com/BleckWolf25/BeaudyShell"
        LANGUAGES C)

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
add_executable(BeaudyShell ${SOURCES})
target_include_directories(BeaudyShell PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)

# Installation configuration
include(GNUInstallDirs)
install(TARGETS BeaudyShell
        RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
        COMPONENT runtime)

# Install documentation
install(FILES 
        "${CMAKE_CURRENT_SOURCE_DIR}/README.md"
        "${CMAKE_CURRENT_SOURCE_DIR}/LICENSE"
        DESTINATION ${CMAKE_INSTALL_DOCDIR}
        COMPONENT docs)

# Packaging configuration
set(CPACK_PACKAGE_NAME "BeaudyShell")
set(CPACK_PACKAGE_VENDOR "BleckWolf25")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "A fast, interactive Unix shell implementation")
set(CPACK_PACKAGE_VERSION_MAJOR ${PROJECT_VERSION_MAJOR})
set(CPACK_PACKAGE_VERSION_MINOR ${PROJECT_VERSION_MINOR})
set(CPACK_PACKAGE_VERSION_PATCH ${PROJECT_VERSION_PATCH})
set(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_CURRENT_SOURCE_DIR}/LICENSE")
set(CPACK_RESOURCE_FILE_README "${CMAKE_CURRENT_SOURCE_DIR}/README.md")

# macOS specific configuration
if(APPLE)
    set(CPACK_GENERATOR "productbuild")
    set(CPACK_SYSTEM_NAME "macOS")
    
    # Create a scripts directory in the build folder
    file(MAKE_DIRECTORY ${CMAKE_BINARY_DIR}/scripts)
    
    # Create a postinstall script
    file(WRITE ${CMAKE_BINARY_DIR}/scripts/postinstall "
#!/bin/bash
# Add BeaudyShell to /etc/shells
if ! grep -q '/usr/local/bin/BeaudyShell' /etc/shells; then
    echo '/usr/local/bin/BeaudyShell' >> /etc/shells
fi

# Create symlink in /usr/local/bin
ln -sf '/Applications/BeaudyShell.app/Contents/MacOS/BeaudyShell' '/usr/local/bin/BeaudyShell'
")
    
    # Make the postinstall script executable
    execute_process(COMMAND chmod +x ${CMAKE_BINARY_DIR}/scripts/postinstall)
    
    # Configure the productbuild
    set(CPACK_PRODUCTBUILD_IDENTIFIER "com.bleckwolf25.beaudyshell")
    set(CPACK_PRODUCTBUILD_SCRIPTS ${CMAKE_BINARY_DIR}/scripts)
endif()

# Linux specific configuration
if(UNIX AND NOT APPLE)
    set(CPACK_GENERATOR "DEB;RPM")
    
    # DEB package settings
    set(CPACK_DEBIAN_PACKAGE_MAINTAINER "BleckWolf25")
    set(CPACK_DEBIAN_PACKAGE_SECTION "shells")
    set(CPACK_DEBIAN_PACKAGE_DEPENDS "libc6 (>= 2.14)")
    
    # RPM package settings
    set(CPACK_RPM_PACKAGE_LICENSE "MIT")
    set(CPACK_RPM_PACKAGE_GROUP "System Environment/Shells")
    set(CPACK_RPM_PACKAGE_REQUIRES "glibc >= 2.14")
endif()

# Component configuration
set(CPACK_COMPONENTS_ALL runtime docs)
set(CPACK_COMPONENT_RUNTIME_DISPLAY_NAME "BeaudyShell Runtime")
set(CPACK_COMPONENT_DOCS_DISPLAY_NAME "Documentation")
set(CPACK_COMPONENT_RUNTIME_DESCRIPTION "BeaudyShell executable and required files")
set(CPACK_COMPONENT_DOCS_DESCRIPTION "Documentation files")

include(CPack)

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