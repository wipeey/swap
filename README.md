# swap

A simple, safe, and robust CLI tool written in Rust to swap two files or directories on Linux.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Description

`swap` is a command-line utility that allows you to interchange two items on your filesystem. It offers two modes of operation:

1.  **Location Swap (default):** Swaps the directories of two items while they keep their original names.
2.  **Name Swap (with `-n` flag):** Swaps the names of two items while they remain in their original directories.

The tool is built with safety as a primary concern:
- It uses atomic rename operations, which are safer and faster than copy/delete.
- It prevents dangerous operations, such as swapping a directory with one of its own subdirectories.
- It provides clear, user-friendly error messages.
- It resolves all paths to their absolute, canonical form before operating to avoid ambiguity.

## Installation

### Build from source
Ensure you have the Rust toolchain installed.

```bash
# 1. Clone the repository
git clone https://github.com/cyprienbf/swap.git
cd swap

# 2. Build the project in release mode
cargo build --release

# 3. The binary will be in `target/release/swap`
# You can move it to a directory in your PATH, e.g.:
sudo mv target/release/swap /usr/local/bin/
```

## Usage

```
$ swap --help
A robust CLI tool to swap two files or directories on Linux.

Usage: swap [OPTIONS] <PATH1> <PATH2>

Arguments:
  <PATH1>  The first path to swap
  <PATH2>  The second path to swap

Options:
  -n, --name-swap  Swap names instead of locations. If this flag is present, items will be renamed to each other but stay in their original directories. By default, items are moved to each other's directories, keeping their original names
  -h, --help       Print help
  -V, --version    Print version
```

## Examples

### 1. Swap Locations (Default Behavior)

Let's swap the location of `report.txt` and `archive.zip`.

**Before:**
```
.
├── project_a/
│   └── report.txt
└── project_b/
    └── archive.zip
```

**Command:**
```bash
swap project_a/report.txt project_b/archive.zip
```

**After:**
```
.
├── project_a/
│   └── archive.zip
└── project_b/
    └── report.txt
```

### 2. Swap Names (using `-n` or `--name-swap` flag)

Let's swap the names of two photos in the same directory.

**Before:**
```
.
└── vacation_photos/
    ├── img_001.jpg
    └── img_002.jpg
```

**Command:**
```bash
swap -n vacation_photos/img_001.jpg vacation_photos/img_002.jpg
```

**After:**
```
.
└── vacation_photos/
    ├── img_002.jpg
    └── img_001.jpg
```

### 3. Handling Errors

The tool will safely exit if an operation is invalid.

**Trying to swap a file that doesn't exist:**
```bash
$ swap file_that_exists.txt file_that_does_not_exist.txt
Error: Path not found: 'file_that_does_not_exist.txt'
```

**Trying to swap a directory into itself:**
```bash
$ swap my_folder my_folder/sub_folder
Error: Cannot swap a directory with its own subdirectory. This is a safety prevention.
```

## License

This project is licensed under the MIT License.
