# Vayload Kit 
![Version](https://img.shields.io/badge/version-1.0.1--alpha.5-blue) ![Rust](https://img.shields.io/badge/rust-1.92.0-orange?logo=rust) ![Platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue) ![GitHub stars](https://img.shields.io/github/stars/vayload/vayload-kit) ![GitHub issues](https://img.shields.io/github/issues/vayload/vayload-kit)

Vayload Kit (vk) is a modern, modular, and secure CLI for creating, managing, and publishing Vayload plugins.


## Installation

### macOS / Linux / WSL / Git Bash

```bash
curl -fsSL https://raw.githubusercontent.com/vayload/vayload-kit/main/scripts/install.sh | bash
```

---

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/vayload/vayload-kit/main/scripts/install.ps1 | iex
```

If execution policy blocks the script:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/vayload/vayload-kit/main/scripts/install.ps1 | iex"
```

---

### Manual Download

You can also download binaries directly from:

[https://github.com/vayload/vayload-kit/releases](https://github.com/vayload/vayload-kit/releases)

---

## After Install

Restart your terminal and run:

```bash
vk
```

---

## Supported Platforms

- macOS (x86_64, arm64)
- Linux (x86_64, arm64)
- Windows (x86_64, arm64)

---

## Project Initialization and Creation

### `vk init`

Initialize a Vayload project in the current directory interactively.

**Usage:** `vk init`

**Arguments:**
- `--yes`: Skip interactive prompts and use default settings.

---

## Dependency Management

### `vk add <package>`

Add a dependency to the `vayload.toml` file and install it.

**Arguments:**
- `--dev`: Install as a development dependency (`dev-dependencies`).
- `<package>@<version>`: Specifies an exact version (e.g., `serde@1.0.130`).

**Example:** `vk add hello-world --dev`

### `vk install`

Install all dependencies listed in the `package.json5` manifest.

**Options:**
- `--offline`: Attempt to install only from local cache without network.
- `--frozen`: Fail if the lockfile needs updating (ideal for CI/CD).

### `vk remove <package>`

Remove a package from the manifest and delete local artifacts.

**Usage:** `vk remove <package-name>`

### `vk update`

Update dependencies according to semver rules.

**Arguments:**
- `<package>`: (Optional) Update only the specified package. If omitted, updates all.

---

## Authentication and Registry

| Command | Description |
| --- | --- |
| `vk login` | Start authentication flow (auth with password and token or oauth: google, github). |
| `vk logout` | Close session and securely delete encrypted credentials. |
| `vk whoami` | Show the currently authenticated user on the registry. |

---

## Publishing and Distribution

### `vk publish`

Upload your package to the official Vayload registry.

**Options:**
- `--tag <name>`: Publish with a specific tag (e.g., `beta`, `next`).
- `--dry-run`: Simulate publishing and show which files would be uploaded without actually uploading.
- `--access <public|private>`: Set package visibility.

### `vk list`

Display a tree of all installed dependencies.

**Options:**
- `--depth <n>`: Limit the depth of the tree shown (e.g., `--depth 1`).

---

## Maintenance and Auditing

| Command | Function |
| --- | --- |
| `vk audit` | Scan the dependency tree for known vulnerabilities. |
| `vk clean` | Free up disk space by removing local cache and build artifacts. |

---

## Links

- Documentation: [https://vayload.dev/docs](https://vayload.dev/docs)
- Issues: [https://github.com/vayload/vayload-kit/issues](https://github.com/vayload/vayload-kit/issues)

---

© Vayload
