# Contributing to BeaudyShell

Thank you for your interest in contributing to BeaudyShell! This document provides guidelines and instructions for contributing.

## Development Setup

1. Fork and clone the repository:

    ```bash
        git clone https://github.com/YourUsername/BeaudyShell.git
        cd BeaudyShell
    ```

2. Create a build directory:

    ```bash
        mkdir build && cd build
    ```

3. Configure and build:

    ```bash
        cmake -DCMAKE_BUILD_TYPE=Debug ..
        cmake --build .
    ```

## Code Style

- Use consistent indentation (4 spaces)
- Follow the existing code style
- Add comments for complex logic
- Include documentation for public functions
- Keep functions focused and small
- Use meaningful variable and function names

## Pull Request Process

1. Create a new branch for your feature/fix
2. Write clear commit messages
3. Update documentation if needed
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request with a clear description

## Reporting Issues

When reporting issues, please include:

- BeaudyShell version
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Any relevant error messages

## Feature Requests

Feature requests are welcome! Please provide:

- Clear description of the feature
- Use cases
- How it benefits the project
- Possible implementation approach
