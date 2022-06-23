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
fpcli 0.2.0
A CLI app for Flatpak manifests.

USAGE:
    fpcli <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    bootstrap         Creates a new application manifest for the current project
    convert           Converts a manifest. The manifest must be a valid Flatpak manifest
    get-type          Get the type of the manifest
    get-urls          Get all the urls contained in a manifest
    help              Print this message or the help of the given subcommand(s)
    is-reverse-dns    Test if a file path uses a reverse DNS ID
    lint              Formats a Flatpak manifest
    ls                List all the Flatpak manifests in a specific directory
    parse             Parse a Flatpak manifest
    resolve           Resolve all the imported manifests in a manifest file
    to-reverse-dns    Converts a URL to its reverse DNS equivalent
    tree              Print the modules of a manifest in a tree-like structure
```

## License
MIT
