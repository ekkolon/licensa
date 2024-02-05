# Licensa

Licensa is a powerful CLI tool designed for seamless source code license management. Developers can effortlessly **verify**, **apply**, **modify**, and **enforce** SPDX license headers across their source code.

As of today, Licensa boasts compatibility with over **65 file types**, making it a versatile and comprehensive solution for license management in diverse coding environments.

## Installation

### Linux

Licensa currently supports the following architectures:

- **x86_64** (64-bit Intel/AMD processors)
- **aarch64** (64-bit ARM processors)

Run the following command and follow the on-screen instructions:

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/ekkolon/licensa/main/scripts/install.sh)"
```

This script will:

- automatically detect your machine's architecture
- download and unpack the necessary .tar
- copy the `licensa` binary to `usr/local/bin`

For more details, you can review the [installation script](/scripts/install.sh) directly.

### Windows

Before running the installation script, ensure that your PowerShell execution policy allows (remote) script execution. You can set the execution policy using the following command:

```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

Executing the following script will walk you through the installation process (you may need to run this command from an elevated shell):

```powershell
Invoke-Expression -Command (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/ekkolon/licensa/main/scripts/install.ps1" -UseBasicParsing).Content
```
