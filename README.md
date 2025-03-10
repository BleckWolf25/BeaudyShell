# BeaudyShell

A fast, interactive Unix shell implementation written in C.

## Features

- Command execution using fork/exec
- Pipes and file redirection (e.g., `cmd1 | cmd2`, `cmd > file`, `cmd < file`)
- Built-in commands (`cd`, `exit`, `pwd`, `help`, `echo`)
- Colorful prompt showing username, hostname, and current directory
- Clean, modular codebase following modern C practices

## Building

```bash
make
```

For debug build with additional information:

```bash
make debug
```

## Usage

```bash
./BeaudyShell
```

## Project Structure

- `main.c` - Entry point
- `shell.c` - Core shell functions
- `input.c` - Input handling and parsing
- `execute.c` - Command execution
- `builtins.c` - Built-in command implementations
- `prompt.c` - Shell prompt handling
- `pipes.c` - Pipe and redirection implementation
- `signals.c` - Signal handling
- `utils.c` - Utility functions

## Roadmap

1. ✅ Core shell functionality (fork/exec/wait)
2. ✅ File redirection and pipes
3. ⬜ Job control and signal handling
4. ⬜ Variable expansion and quoting
5. ⬜ Interactive features (history, tab completion)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
