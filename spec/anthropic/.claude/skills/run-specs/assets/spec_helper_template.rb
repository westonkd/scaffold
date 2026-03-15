# Minimal spec_helper.rb template for Rails + RSpec projects.
# Copy to spec/spec_helper.rb and adjust as needed.

require "rails_helper"

RSpec.configure do |config|
  # Only run examples tagged :focus when any exist.
  config.filter_run_when_matching :focus

  # Re-run failures first, then the rest in random order.
  config.order = :random
  Kernel.srand config.seed

  # Allow `expect` syntax only (no `should`).
  config.expect_with :rspec do |expectations|
    expectations.include_chain_clauses_in_custom_matcher_descriptions = true
    expectations.syntax = :expect
  end

  # Persist failure state for --only-failures / --next-failure.
  config.example_status_persistence_file_path = "tmp/rspec-status.txt"

  config.mock_with :rspec do |mocks|
    mocks.verify_partial_doubles = true
  end

  config.shared_context_metadata_behavior = :apply_to_host_groups
end
