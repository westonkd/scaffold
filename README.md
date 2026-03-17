# 🏗️ scaffold

A collection of tools for composing multi-repo development environments and managing AI agent artifacts.

> [!IMPORTANT]
> Scaffold tools are experimental. They represent patterns I'm testing and refining.

## Tools

### hoist

Hoist AI agent artifacts (Claude Code skills, etc.) from repos into your current directory.

#### Installation

```bash
brew install westonkd/hoist/hoist
```

Or build from source:

```bash
cargo build --release
cp target/release/hoist ~/.local/bin/
```

#### Usage

##### With a path argument

```bash
hoist ./some-repo
```

##### With a `hoist.json` config

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

##### Removing hoisted artifacts

Remove all artifacts from a specific repo:

```bash
hoist unhoist ./some-repo
```

Or prune any artifacts whose source is no longer listed in `hoist.json`:

```bash
hoist unhoist
```

Pass `--dry-run` to preview what would be removed without touching anything:

```bash
hoist unhoist --dry-run
```

#### The Problems

**1. Agents don't reliably discover nested artifacts.**
LLM agents often miss skills, `AGENTS.md` files, and other artifacts when they're buried in sub-directories across many repos. `hoist` surfaces them to a common root where agents can find them consistently.

**2. Artifacts belong near the code, but may need proprietary context.**
The most useful agent artifacts are specific to a codebase — but adding proprietary instructions or context to an open-source repo isn't always appropriate. With `hoist`, you can keep those artifacts in a separate closed-source repo and hoist them alongside the OSS code at runtime, keeping source and context cleanly separated.

#### Documentation

- [Command reference](docs/commands.md)
- [How it works in depth](docs/how-it-works.md)
