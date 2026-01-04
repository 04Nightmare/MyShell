[![progress-banner](https://backend.codecrafters.io/progress/shell/29b15116-e721-4e6c-9194-75bdcccaa607)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)
Guided by the team of [codecrafters.io](https://codecrafters.io)
#  >< Rust Shell >< 

This project is a custom shell program implemented in **Rust**, designed to mimic basic shell operations in Unix/Linux systems. The shell supports essential features such as **basic built-in commands, external command execution, piping, redirection, autocomplete environment variable handling and history persistence**. 

This project is developed **step-by-step** from scratch and helps in understanding *how shells work and how processes interact* with the operating system.

---

## ✨ Features

### ✅ Core Command Execution
- Execute external commands via `$PATH`
- Built-in commands:
  - `echo`
  - `cd`
  - `pwd`
  - `type`
  - `exit`
  - `history`

---

### 📜 History Management
- In-memory command history
- Plain-text history persistence (POSIX-style)
- Supports:
  - `history` — show full history
  - `history N` — show last `N` entries (with correct numbering)
  - `history -r FILE` — read history from file
  - `history -w FILE` — write history to file
  - `history -a FILE` — append new histories to file
- Can accept `$HISTFILE` on startup
- Closely matches **bash-style numbering and behavior for history display**

---
### ⏩ Redirection
- Supports basic POSIX-style I/O redirection with correct file descriptor handling.
- ✔️ Supported Operators:
  - `> | 1>` — redirect **stdout** (overwrite)
  - `>>` — redirect **stdout** (append)
  - `2>` — redirect **stderr** (overwrite)
  - `2>>` — redirect **stderr** (append)
- Redirections are applied before command execution
- Output files are created if they do not exist
- Works with both **built-in** and **external** commands
---

### 🔗 Pipelines
- Supports **multi-command pipelines**:
  ```sh
  cmd1 | cmd2 | cmd3 | ...


## 🛠️ Installation
### Clone the repository
```
git clone (https://github.com/04Nightmare/MyShell.git)
cd MyShell
```
### Compile Shell
```
cargo build
```
OR (for optimized build)
```
cargo build --release
```
### Run the shell
```
cargo run
```
Or run the release binary directly:
```
./target/release/codecrafters-shell
```
Add to path
```
sudo cp ./target/release/codecrafters-shell /usr/local/bin
codecrafters-shell
```