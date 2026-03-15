---
name: run-specs
description: Run the RSpec suite for a file, directory, or the whole project. Use when the user asks to run specs, tests, or RSpec, or when you want to verify code changes with tests.
compatibility: Requires Ruby and bundler. Designed for Rails/RSpec projects.
allowed-tools: Bash
---

Run the RSpec suite and report results.

## Instructions

1. If a target path was provided, run:
   ```
   bundle exec rspec <target>
   ```
   Otherwise run the full suite:
   ```
   bundle exec rspec spec/
   ```

2. Parse the output for failures. For each failure, report:
   - The example description
   - The file and line number
   - The failure message and diff (if present)

3. If all examples pass, confirm with the summary line (e.g. `47 examples, 0 failures`).

4. Do not attempt to fix failures unless the user explicitly asks.

## Resources

- Run via `scripts/run.sh [target]` — writes a JSON results file to `tmp/rspec-results.json` in addition to terminal output.
- See `references/REFERENCE.md` for useful flags, single-example syntax, parallel runs, and common failure patterns.
- See `assets/spec_helper_template.rb` for a recommended `spec/spec_helper.rb` starting point.
