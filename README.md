# hoist

Hoist AI agent artifacts (Claude Code skills, etc.) from repos into your current directory.

**hoist is experimental!** This is a pattern I'm iterating on.

## The Problems

**1. Agents don't reliably discover nested artifacts.**
LLM agents often miss skills, `AGENTS.md` files, and other artifacts when they're buried in sub-directories across many repos. `hoist` surfaces them to a common root where agents can find them consistently.

**2. Artifacts belong near the code, but may need proprietary context.**
The most useful agent artifacts are specific to a codebase — but adding proprietary instructions or context to an open-source repo isn't always appropriate. With `hoist`, you can keep those artifacts in a separate closed-source repo and hoist them alongside the OSS code at runtime, keeping source and context cleanly separated.

## Usage

### With a `hoist.json` config

Create `hoist.json` in your working directory:

```json
{
  "roots": [
    "./canvas-lms",
    "./my-other-repo"
  ]
}
```

Then run:

```bash
hoist
```

### With a path argument

```bash
hoist ./some-repo
```

## Installation

```bash
cargo build --release
cp target/release/hoist ~/.local/bin/
```

## Documentation

- [Command reference](docs/commands.md)
- [How it works in depth](docs/how-it-works.md)
