# Licensa

[![codecov](https://codecov.io/gh/ekkolon/licensa/graph/badge.svg?token=APBWVQY4HF)](https://codecov.io/gh/ekkolon/licensa)
![Crates.io Version](https://img.shields.io/crates/v/licensa)
![docs.rs](https://img.shields.io/docsrs/licensa)
![Crates.io License (version)](https://img.shields.io/crates/l/licensa/0.1.0)

Licensa is a powerful CLI tool designed for seamless source code license management. Developers can effortlessly **verify**, **apply**, **modify**, and **enforce** SPDX license headers across their source code.

As of today, Licensa boasts compatibility with over **65 file types**, making it a versatile and comprehensive solution for license management in diverse coding environments.

## Table of Contents

- [CLI Reference](#cli-reference)
- [Installation](#installation)
- [License](#license)
- [Contribution](#contribution)

## CLI Reference

### Commands

#### `init`

Generate project-wide Licensa configuration.

```bash
licensa init [OPTIONS]
```

#### `apply`

Add SPDX copyright license headers to source code files.

```bash
licensa apply [OPTIONS]
```

#### `verify`

Verify presence of license headers in source code files.

```bash
licensa verify
```

### Options

**-t, --type** `required` SPDX License identifier (case-insensitive)

Optional, if present in .licensarc config file

*default*: N/A

**-o, --owner** `required` The copyright owner

Optional, if present in .licensarc config file

*default*: N/A

**--year** The copyright year or a range of years

*default*: null

**--exclude** A list of glob patterns to exclude from the licensing process

*default*: [ ]

**--email** The E-Mail of the copyright owner

*default*: null

**--project** The name of the project to be licensed
  
*default*: null

**--project-url** URL of the project

*default*: null

#### Globals

**--verbose** Detailed command output

**-h, --help** Show help message

**-v, --version** Show program's version number

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

## License

Licensed under either of

- [MIT license](https://spdx.org/licenses/MIT.html) (see [LICENSE-MIT](/LICENSE-MIT)) or
- [Apache License, Version 2.0](https://spdx.org/licenses/Apache-2.0.html) (see [LICENSE-APACHE](/LICENSE-APACHE))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
