# Licensa

Licensa is a powerful CLI tool designed for seamless source code license management. Developers can effortlessly **verify**, **apply**, **modify**, and **enforce** SPDX license headers across their source code.

As of today, Licensa boasts compatibility with over **65 file types**, making it a versatile and comprehensive solution for license management in diverse coding environments.

## Installation

### Windows

Before running the installation script, ensure that your PowerShell execution policy allows (remote) script execution. You can set the execution policy using the following command:

```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

Executing the following script will walk you through the installation process (you may need to run this command from an elevated shell):

```powershell
Invoke-Expression -Command (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/ekkolon/licensa/main/scripts/install.ps1" -UseBasicParsing).Content
```
