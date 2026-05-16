```
╔════════════════════════════════════════════════════════════╗
║                                                            ║
║            ███╗   ███╗██╗███╗   ███╗██╗██████╗             ║
║            ████╗ ████║██║████╗ ████║██║██╔══██╗            ║
║            ██╔████╔██║██║██╔████╔██║██║██████╔╝            ║
║            ██║╚██╔╝██║██║██║╚██╔╝██║██║██╔══██╗            ║
║            ██║ ╚═╝ ██║██║██║ ╚═╝ ██║██║██║  ██║            ║
║            ╚═╝     ╚═╝╚═╝╚═╝     ╚═╝╚═╝╚═╝  ╚═╝            ║
║                                                            ║
║            Git-inspired Version Control CLI                ║
║                                                            ║
╚════════════════════════════════════════════════════════════╝
```

A Git-inspired version control system written in Rust that recreates core Git internals - blobs, trees, commits, object storage, hashing, compression, and history traversal through a command-line interface.

## Overview

Mimir is a minimal VCS that replicates Git's object storage model. It was built to understand how Git works under the hood using object hashing, zlib compression, tree traversal, and commit chaining through a clean CLI.

It includes both low-level and high-level commit workflows:
- `commit-tree` creates a commit object directly from a provided tree hash
- `commit` creates a commit from the current working tree and updates the current branch reference

## Features

- Initialize a repository structure
- Store file contents as Git-style blob objects
- Inspect stored objects with `cat-file`
- Build tree objects from the working directory
- Create raw commit objects with `commit-tree`
- Create branch-aware commits with `commit`
- Traverse commit history with `log`
- Persist compressed objects in `.mimir/objects`

## Installation

**Requirements:**
- Rust (stable)
- Cargo

**Setup:**
```bash
git clone https://github.com/omanshchoudhary/mimir
cd mimir
cargo build --release
```

## Quick Start

Use Cargo during development:

```bash
# Start fresh and initialize the repository
rm -rf .mimir
cargo run -- init

# Create a sample file
printf "Hello from Mimir\n" > hello.txt

# Hash the file into the object store
cargo run -- hash-object hello.txt

# Build a tree object from the current working directory
cargo run -- write-tree

# Create a commit from the current working tree
cargo run -- commit -m "Initial commit"

# View commit history
cargo run -- log
```

If you want to test the lower-level plumbing command directly, first capture a tree hash and then create a commit from it:

```bash
TREE_HASH=$(cargo run -q -- write-tree)
cargo run -- commit-tree "$TREE_HASH" -m "Commit created from tree hash"
```

After building a release binary, you can also run commands as:

```bash
./target/release/mimir <command>
```

## Usage

### Init
Create a new `.mimir` repository structure.

```bash
cargo run -- init
```
**Example output:**
```
Initialized empty mimir repo
```

### Hash Object
Store a file as a Git-style blob object.

```bash
cargo run -- hash-object test.txt
```
**Example output:**
```
8ab686eafeb1f44702738c8b0f24f2567c36da6d
```

### Cat File
Read and print the contents of a stored object, along with its compressed size and storage path.

```bash
cargo run -- cat-file -p 8ab686eafeb1f44702738c8b0f24f2567c36da6d
```
**Example output:**
```
Hello, world
32 .mimir/objects/8a/b686eafeb1f44702738c8b0f24f2567c36da6d
```

### Write Tree
Build a tree object from the current working directory.

```bash
cargo run -- write-tree
```
**Example output:**
```
4b825dc642cb6eb9a060e54bf8d69288fbee4904
```

### Commit Tree
Create a commit object directly from a tree hash.

```bash
cargo run -- commit-tree <tree_hash> -m "Add initial files"
```
**Example output:**
```
3f6e1d9a...
```

### Commit
Create a commit from the current working tree and update the current branch.

```bash
cargo run -- commit -m "Add initial files"
```
**Example output:**
```
Committed to branch main: 3f6e1d...
```

### Log
Traverse and print commit history from the current branch tip.

```bash
cargo run -- log
```
**Example output:**
```
commit 3f6e1d9a...

    Add initial files

commit 1a2b3c4d...

    Initial commit
```

## Project Structure

```
mimir/
├── src/
│   └── main.rs          # CLI commands and object logic
├── .mimir/
│   ├── HEAD             # Active branch reference
│   ├── objects/         # Compressed object store
│   └── refs/heads/      # Branch tip hashes
├── Cargo.toml           # Dependencies and package metadata
└── Cargo.lock
```

## Object Format

Objects use Git-style headers followed by raw content, then are compressed with zlib:

```
blob <size>\0<content>
tree <size>\0<mode> <name>\0<20-byte-hash>...
commit <size>\0tree <hash>\nparent <hash>?\nauthor ...\ncommitter ...\n\n<message>
```

## Tech Stack

- **Rust** - core implementation
- **Clap** - CLI argument parsing
- **Flate2** - zlib compression/decompression
- **SHA-1** - content-addressed hashing
- **Hex** - hash encoding