# Create build directory
mkdir -p build && cd build

# Configure with CMake
cmake ..

# Build the project
cmake --build .

# Create the macOS installer (.dmg)
cpack -G DragNDrop