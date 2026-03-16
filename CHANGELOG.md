# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2026-03-16

### Added

- `anthropic/marketplace` hoist strategy for Anthropic marketplace artifacts
- `anthropic/plugin` hoist strategy for Anthropic plugin artifacts

## [0.1.2] - 2026-03-15

### Fixed

- Use authenticated token when pushing to Homebrew release repo

## [0.1.1] - 2026-03-15

### Fixed

- Update release workflow to correctly target the `hoist` package

## [0.1.0] - 2026-03-15

### Added

- Initial release of the `hoist` CLI tool
- `anthropic/claude_code/agent_skills` strategy to hoist Claude Code agent skills
- Support for directory-based agent skills
- `update` subcommand to refresh previously hoisted artifacts
- Homebrew distribution via release workflow
