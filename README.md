# Licensa

[![codecov](https://codecov.io/gh/ekkolon/licensa/graph/badge.svg?token=APBWVQY4HF)](https://codecov.io/gh/ekkolon/licensa)
![Crates.io Version](https://img.shields.io/crates/v/licensa)
![docs.rs](https://img.shields.io/docsrs/licensa)
![Crates.io License (version)](https://img.shields.io/crates/l/licensa/0.1.0)

Licensa is a powerful CLI tool designed for seamless source code license management. Developers can effortlessly **verify**, **apply**, **modify**, and **enforce** SPDX license headers across their source code.

As of today, Licensa boasts compatibility with over **65 file types**, making it a versatile and comprehensive solution for license management in diverse coding environments.

You may be looking for:

- [Overview](#overview)
- [Installation](#installation)
- [Usage](#usage)
- [CLI Reference](#cli-reference)
- [License](#license)
- [Contribution](#contribution)

## Overview

Explore the [complete CLI reference](#cli-reference) for a comprehensive list of [options](#options).

**`init`** Generate project-wide Licensa configuration.

```bash
licensa init [OPTIONS]
```

**`apply`** Add SPDX copyright license headers to source code files.

```bash
licensa apply [OPTIONS]
```

**`verify`** Verify presence of license headers in source code files.

```bash
licensa verify
```

## Installation

### Cargo

```shell
cargo install licensa
```

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

### Windows (PowerShell)

Before running the installation script, ensure that your PowerShell execution policy allows (remote) script execution. You can set the execution policy using the following command:

```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

Executing the following script will walk you through the installation process (you may need to run this command from an elevated shell):

```powershell
Invoke-Expression -Command (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/ekkolon/licensa/main/scripts/install.ps1" -UseBasicParsing).Content
```

## Usage

> This is an example walkthrough

Suppose you have a TypeScript project "Lord Of The Rings Map" (lotr-map) with the following initial structure:

```bash
.
└── lotr-map/
    ├── node_modules/
    ├── src/
    │   ├── map/
    │   │   └── Terrain.ts    # has license header
    │   ├── Rivendell.ts      # no license header
    │   ├── Isengard.ts       # no license header
    │   ├── Mithrandir.ts     # has license header
    │   ├── Gondor.ts         # has license header
    │   └── MinasTirith.ts    # has license header
    ├── .gitignore  
    ├── package.json
    └── tsconfig.json
```

### Generate project-wide Licensa configurations

> When a `.licensarc` config file is present, required arguments in subsequent commands become optional if they are already defined in the config file.

Licensa includes a pre-built [.licensaignore](/.licensaignore) file, akin to a *.gitignore*, allowing you to specify patterns for including/excluding files from the licensing process.

To generate project-wide configurations, execute the following command:

```bash
licensa init --type Apache-2.0 --owner "Eru Ilúvata" --year 2001
```

Upon successful execution, the command will expand your project structure to include the *.licensarc* and [.licensaignore](/.licensaignore):

```bash
.
└── lotr-map/
    ├── node_modules/
    ├── src/
    │   ├── map/
    │   │   └── Terrain.ts    # has license header
    │   ├── Rivendell.ts      # no license header
    │   ├── Isengard.ts       # no license header
    │   ├── Mithrandir.ts     # has license header
    │   ├── Gondor.ts         # has license header
    │   └── MinasTirith.ts    # has license header
    ├── .gitignore  
    ├── .licensaignore        # ADDED
    ├── .licensarc            # ADDED
    ├── package.json
    └── tsconfig.json
```

The generated `.licensarc` config file would contain the following settings:

```json
{
  "type": "Apache-2.0",
  "owner": "Eru Ilúvata",
  "year": 2001
}
```

### Add license headers to your source files

Expanding on the project layout described earlier, it's noted that three source files already have SPDX license headers applied to them. Consequently, these files are excluded from the licensing process.

#### With defaults from `.licensarc` config file

```bash
licensa apply
```

##### *src/Rivendell.ts*

```ts
/**
 * Copyright 2001 Eru Ilúvata
 * SPDX-License-Identifier: Apache-2.0
 */

import Terrain from './map/Terrain.ts';

export class Rivendell extends Terrain {}
```

##### *src/Isengard.ts*

```ts
/**
 * Copyright 2001 Eru Ilúvata
 * SPDX-License-Identifier: Apache-2.0
 */

import Terrain from './map/Terrain.ts';

export class Isengard extends Terrain {}
```

#### Override defaults with supplied options

```bash
licensa apply --type AGPL-3.0-only --owner Sauron --year "2002-present"
```

##### *src/Rivendell.ts*

```ts
/**
 * Copyright 2002-present Sauron
 * SPDX-License-Identifier: AGPL-3.0-only
 */

import Terrain from './map/Terrain.ts';

export class Rivendell extends Terrain {}
```

##### *src/Isengard.ts*

```ts
/**
 * Copyright 2002-present Sauron
 * SPDX-License-Identifier: AGPL-3.0-only
 */

import Terrain from './map/Terrain.ts';

export class Isengard extends Terrain {}
```

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

#### Globals

**--verbose** Detailed command output

**-h, --help** Show help message

**-v, --version** Show program's version number

## License

Licensed under either of

- [MIT license](https://spdx.org/licenses/MIT.html) (see [LICENSE-MIT](/LICENSE-MIT)) or
- [Apache License, Version 2.0](https://spdx.org/licenses/Apache-2.0.html) (see [LICENSE-APACHE](/LICENSE-APACHE))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
