# RSpec Reference

## Useful flags

| Flag | Description |
|---|---|
| `--only-failures` | Re-run only examples that failed on the last run |
| `--next-failure` | Run the next failure, then stop |
| `--fail-fast` | Stop after the first failure |
| `--format documentation` | Show full example descriptions instead of dots |
| `--tag focus` | Run only examples tagged with `:focus` |
| `--order random` | Randomise example order (default); use `--seed N` to replay |
| `--bisect` | Bisect the suite to find the minimal set causing a failure |

## Running a single example

Append the line number of the `it` block:

```bash
bundle exec rspec spec/models/user_spec.rb:42
```

## Focusing examples without a flag

Tag an example or group with `:focus` and run with `--tag focus`, or use the
`fit` / `fdescribe` / `fcontext` aliases (requires `config.filter_run_when_matching :focus`).

## Parallel runs

If the project uses `parallel_tests`:

```bash
bundle exec parallel_rspec spec/
```

## Common failure patterns

| Symptom | Likely cause |
|---|---|
| `Failure/Error: … ActiveRecord::StatementInvalid` | Database not migrated — run `bin/rails db:test:prepare` |
| Random failures that disappear on re-run | Order-dependent state — use `--bisect` to isolate |
| `NameError: uninitialized constant` | Missing require or autoload — check `spec_helper.rb` |
| Extremely slow suite | Missing `use_transactional_fixtures` or heavy `before(:all)` hooks |
