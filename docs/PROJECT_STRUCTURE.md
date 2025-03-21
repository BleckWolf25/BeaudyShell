# PROJECT STRUCTURE - ADVANCED

```zsh
.
├── include/                    # Header files
│   ├── beaudyshell.h           # Main header with core definitions
│   ├── builtins.h              # Built-in commands interface
│   ├── config.h                # Shell configuration options
│   ├── execute.h               # Command execution handling
│   ├── input.h                 # Input processing
│   ├── jobs.h                  # Job control system
│   ├── pipes.h                 # Pipeline handling
│   ├── prompt.h                # Shell prompt customization
│   ├── signals.h               # Signal handling
│   └── utils.h                 # Utility functions
│
├── src/                        # Source files
│   ├── main.c                  # Entry point
│   ├── shell.c                 # Core shell operations
│   ├── input.c                 # Input reading and parsing
│   ├── execute.c               # Command execution logic
│   ├── builtins.c              # Built-in command implementations
│   ├── job_builtins.c          # Job control commands (fg, bg, jobs)
│   ├── jobs.c                  # Job management system
│   ├── pipes.c                 # Pipeline implementation
│   ├── prompt.c                # Prompt formatting and display
│   ├── signals.c               # Signal handlers
│   └── utils.c                 # Utility functions
│
├── docs/                       # Documentation
│   └── SHELL_FLOW.md           # Shell execution flow diagram
│   └── PROJECT_STRUCTURE.md    # Project structure documentation
│
├── CMakeLists.txt              # Build system configuration
├── CODE_OF_CONDUCT.md          # Project code of conduct
├── CONTRIBUTING.md             # Contribution guidelines
├── LICENSE                     # MIT License
├── README.md                   # Project documentation
└── SECURITY.md                 # Security policy
```

## BeaudyShell Overview

BeaudyShell is a modern Unix shell implementation that combines traditional shell functionality with modern features:

### Core Features

- **Command Execution**: Robust execution of both built-in and external commands
- **Job Control**: Full job management with background processes and signal handling
- **Pipeline Support**: Multi-stage command pipelines with I/O redirection
- **Built-in Commands**: Essential commands (cd, pwd, jobs, fg, bg, etc.)
- **Interactive Interface**: Colorful prompt with user, host, and directory info
- **Speedy & Beauty**: Combines speed and a beautiful interface for terminal amateurs

### Technical Highlights

- Written in C11 for maximum performance, portability & Compatibility.
- Modular architecture with clear separation of concerns
- Comprehensive signal handling and job control
- Cross-platform support (macOS and Linux)
- CMake-based build system with automated installers
