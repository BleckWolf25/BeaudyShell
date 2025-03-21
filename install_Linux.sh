# Create build directory
mkdir -p build && cd build

# Configure with CMake
cmake ..

# Build the project
cmake --build .

# Create DEB package
cpack -G DEB

# Create RPM package
cpack -G RPM