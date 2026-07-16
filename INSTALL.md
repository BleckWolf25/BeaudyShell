# Installation, Uninstallation, and Updating BeaudyShell

## Quick Install (Unix‑like systems)

The repository ships with an **`install.sh`** script that performs a platform‑detect, downloads the latest release binary, verifies its SHA‑256 checksum, and installs it to a location in your `$PATH`.

```bash
# From any directory – it will fetch the newest release automatically
curl -fsSL https://raw.githubusercontent.com/BleckWolf25/BeaudyShell/main/install.sh | bash
```

The script:
1. Detects OS (Linux/macOS) and architecture (x86_64, aarch64).
2. Downloads the matching binary from the GitHub Releases page.
3. Verifies the checksum against the `checksums.txt` file in the release.
4. Installs the binary to `/usr/local/bin/beaudy-shell` (or `$HOME/.local/bin` if you lack sudo).
5. Creates a default configuration file at `~/.beaudy.toml` if none exists.

> **Tip:** If you prefer a manual install, skip the script and follow the *Manual Installation* section below.

## Manual Installation

### 1. Build from source (requires Rust and `make`)

```bash
# Clone the repository
git clone https://github.com/BleckWolf25/BeaudyShell.git
cd BeaudyShell

# Build the release binary
make release   # produces ./target/release/beaudy-shell

# Install the binary (you may need sudo)
sudo install -Dm755 target/release/beaudy-shell /usr/local/bin/beaudy-shell
```

### 2. Download a pre‑compiled binary

1. Visit the **Releases** page on GitHub.
2. Choose the appropriate asset for your OS/arch (e.g. `beaudy-shell-linux-x86_64`).
3. Verify the checksum:
   ```bash
   echo "<checksum>  beaudy-shell-linux-x86_64" | sha256sum -c -
   ```
4. Make it executable and move it to a directory in your `$PATH`:
   ```bash
   chmod +x beaudy-shell-linux-x86_64
   sudo mv beaudy-shell-linux-x86_64 /usr/local/bin/beaudy-shell
   ```

### 3. Windows installation (PowerShell)

1. Download the `beaudy-shell-windows-x86_64.exe` asset from the latest release.
2. Verify the checksum with `Get-FileHash`:
   ```powershell
   Get-FileHash .\beaudy-shell-windows-x86_64.exe -Algorithm SHA256
   ```
3. Rename it to `beaudy-shell.exe` and place it in a folder that is on your `PATH`, e.g. `C:\Program Files\BeaudyShell`.
4. Optionally add the folder to the PATH environment variable:
   ```powershell
   [Environment]::SetEnvironmentVariable('PATH', $env:PATH + ';C:\Program Files\BeaudyShell', 'Machine')
   ```

## Uninstallation

### Unix‑like systems

```bash
# Remove the binary
sudo rm -f /usr/local/bin/beaudy-shell   # or $HOME/.local/bin/beaudy-shell

# Remove the config file (optional)
rm -f $HOME/.beaudy.toml
```

If you used the `install.sh` script with the user‑local fallback (`$HOME/.local/bin`), adjust the path accordingly.

### Windows

1. Delete `beaudy-shell.exe` from the installation folder.
2. Remove the folder from the system `PATH` if you added it manually.
3. Delete the configuration file located at `%USERPROFILE%\.beaudy.toml` (if you created one).

## Updating BeaudyShell

### Using the installer script

Re‑run the same `curl … | bash` command; the script detects an existing installation and overwrites it with the newest version.

### Via Cargo (if you prefer the Rust toolchain)

```bash
cargo install beaudy-shell --force
```

This fetches the latest published crate from `crates.io` (if you publish it) and replaces the existing binary.

### Manual binary replacement

Download the newer binary as described in *Manual Installation* and overwrite the existing executable in your `$PATH`.

## Making BeaudyShell the Default Shell

### Linux / macOS (POSIX shells)

1. **Add the binary to `/etc/shells`** (required by `chsh`):
   ```bash
   echo /usr/local/bin/beaudy-shell | sudo tee -a /etc/shells
   ```
   Adjust the path if you installed to `$HOME/.local/bin`.
2. **Change your default shell**:
   ```bash
   chsh -s $(which beaudy-shell)
   ```
   You will be prompted for your password; the change takes effect on the next login.

### macOS specific

macOS ships with `chsh` as well, but the shell must also be listed in `/etc/shells`. Follow the same steps as above. You can also set the default shell from **System Settings → Users & Groups → Advanced Options**.

### Windows (Terminal Emulators)

- **Windows Terminal**: Add a new profile in `settings.json`:
  ```json
  {
    "name": "BeaudyShell",
    "commandline": "C:\\Program Files\\BeaudyShell\\beaudy-shell.exe",
    "startingDirectory": "%USERPROFILE%"
  }
  ```
- **PowerShell/Command Prompt**: You can set an alias or a shortcut that launches `beaudy-shell.exe`.
- **WSL**: Inside a WSL distro you can use the same POSIX method (`chsh -s $(which beaudy-shell)`).

## Troubleshooting

- **"command not found"** – Ensure the binary directory is in your `$PATH` (`echo $PATH`).
- **Permission denied** – Make sure the binary is executable (`chmod +x …`).
- **Shell refuses to start** – Verify the binary runs directly (`beaudy-shell --version`). If it crashes, run `beaudy-shell` from a regular terminal to see error output.

---

*All commands assume a standard Unix‑like environment; adapt paths as necessary for your setup.*
