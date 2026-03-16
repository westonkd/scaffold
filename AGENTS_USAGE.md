# hoist — Agent Usage Instructions

You are an AI agent. This file contains instructions for you to follow to help your user get set up with hoist.

## What hoist does

hoist symlinks AI agent artifacts (skills, commands, agents, and hooks) from source repositories into the current working directory's `.claude/` folder. It solves two problems:

- **Discoverability**: agents only see artifacts in the working directory — hoist surfaces artifacts from any number of source repos so nothing is missed.
- **Separation of concerns**: proprietary context and tooling can live in a separate closed-source repo alongside open-source code, and hoist bridges them at runtime.

Artifacts are namespaced by repo name, so there are never collisions when multiple repos contribute the same artifact type.

> **Note:** While the examples here use Claude Code, hoist's benefits compound when your repos contain artifacts for multiple agent tools — the more tools in the mix, the more value hoist provides.

## Install hoist

Check whether hoist is already available:

```sh
which hoist
```

If it is not on PATH, install it via Homebrew:

```sh
brew install westonkd/hoist/hoist
```

## Getting started: single repo (no config)

For a single repo, no config file is needed. Run:

```sh
hoist ./path/to/repo
```

hoist detects which artifact types the repo contains and symlinks them into `.claude/` in the current working directory.

## Getting started: multi-repo config

For users with multiple repos, create a `hoist.json` in the working directory:

```json
{
  "roots": [
    "./repo-one",
    "./repo-two"
  ]
}
```

Then run hoist with no arguments:

```sh
hoist
```

hoist reads `hoist.json` and processes all listed repos in one pass.

## Generate a personalized config for the user

Inspect the user's current working directory for sibling directories that look like repos — specifically, directories containing a `.git` folder, a `.claude/` directory, or a `.claude-plugin/` directory. Then present the user with a `hoist.json` tailored to their actual directory layout, for example:

```json
{
  "roots": [
    "../their-repo-one",
    "../their-repo-two"
  ]
}
```

Offer to write this file to the working directory for them.

## Re-running and `--force`

hoist is idempotent — re-run it any time new repos are added or updated:

```sh
hoist
```

Use `--force` to replace existing symlinks and re-merge hooks from scratch:

```sh
hoist --force
```
