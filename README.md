# BeaudyShell

A fast, interactive Unix shell implementation written in C.

## Features

- Command execution using fork/exec
- Pipes and file redirection (e.g., `cmd1 | cmd2`, `cmd > file`, `cmd < file`)
- Built-in commands (`cd`, `exit`, `pwd`, `help`, `echo`)
- Job control and background processes
- Colorful prompt showing username, hostname, and current directory
- Clean, modular codebase following modern C practices

## Installation

### macOS

```bash
# Option 1: Using the installer package
curl -O https://github.com/BleckWolf25/BeaudyShell/releases/latest/download/BeaudyShell.pkg
sudo installer -pkg BeaudyShell.pkg -target /

# Option 2: Building from source
git clone https://github.com/BleckWolf25/BeaudyShell.git
cd BeaudyShell
mkdir build && cd build
cmake ..
cmake --build .
sudo cpack -G productbuild
sudo installer -pkg BeaudyShell.pkg -target /
```

### Linux

#### Debian/Ubuntu

```bash
# Option 1: Using the pre-built package
curl -O https://github.com/BleckWolf25/BeaudyShell/releases/latest/download/beaudyshell_0.1.0_amd64.deb
sudo dpkg -i beaudyshell_0.1.0_amd64.deb

# Option 2: Building from source
git clone https://github.com/BleckWolf25/BeaudyShell.git
cd BeaudyShell
mkdir build && cd build
cmake ..
cmake --build .
cpack -G DEB
sudo dpkg -i beaudyshell_0.1.0_amd64.deb
```

#### Red Hat/Fedora

```bash
# Option 1: Using the pre-built package
curl -O https://github.com/BleckWolf25/BeaudyShell/releases/latest/download/beaudyshell-0.1.0.x86_64.rpm
sudo rpm -i beaudyshell-0.1.0.x86_64.rpm

# Option 2: Building from source
git clone https://github.com/BleckWolf25/BeaudyShell.git
cd BeaudyShell
mkdir build && cd build
cmake ..
cmake --build .
cpack -G RPM
sudo rpm -i beaudyshell-0.1.0.x86_64.rpm
```

## Project Structure (Simplified)

```zsh
.
├── include/        # Header files
├── src/            # Source files
├── docs/           # Documentation
└── CMakeLists.txt  # Build configuration
```

## Development

### Requirements

- CMake 3.10 or higher
- C compiler (GCC/Clang)
- Make or Ninja build system

### Building for Development

```bash
mkdir build && cd build
cmake -DCMAKE_BUILD_TYPE=Debug ..
cmake --build .
```

### Running Tests

```bash
cd build
ctest --output-on-failure
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
