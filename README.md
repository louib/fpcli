# fpcli
A CLI app for Flatpak manifests.

![Code formatting](https://github.com/louib/fpcli/workflows/formatting/badge.svg)
[![dependency status](https://deps.rs/repo/github/louib/fpcli/status.svg)](https://deps.rs/repo/github/louib/fpcli)
[![License file](https://img.shields.io/github/license/louib/fpcli)](https://github.com/louib/fpcli/blob/master/LICENSE)

## Installing
If you already have `cargo` installed, you can install `fpcli` directly from `crates.io`:
```
cargo install fpcli
```

## Usage
```
fpcli 
A CLI app for Flatpak manifests.

USAGE:
    fpcli <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    bootstrap    Creates a new application manifest for the current project
    get-urls     Get all the urls contained in a manifest
    help         Print this message or the help of the given subcommand(s)
    lint         Formats a Flatpak manifest
    ls           List all the Flatpak manifests in a specific directory
    parse        Parse a Flatpak manifest
    resolve      Resolve all the imported manifests in a manifest file
    to-yaml      Converts a manifest to YAML. The manifest must be a valid Flatpak manifest
    tree         Print the modules of a manifest in a tree-like structure
```

## License
MIT
