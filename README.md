# zen 🧘

`zen` is a small Rust CLI tool to keep your local repositories clean and peaceful.

---

## Commands

### `zen sweep`
Recursively deletes all `node_modules` folders under the current directory.

### `zen prune`
Deletes all local Git branches that have been removed from the remote.

### `zen pulse`
Shows contributors ranked by the number of commits in the current repository.

---

## Usage

```bash
zen sweep        # remove all node_modules
zen prune        # clean up stale git branches
zen pulse        # list contributors
```

## Setup

```bash
cargo build
cargo run -- <command>
```

## Installation

```bash
cargo install --path .
```