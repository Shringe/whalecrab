const all_crates_with_tests = [
  "whalecrab_engine"
  "whalecrab_lib"
  "uci"
  "panic_logger"
]
const all_crates_with_canary_tests = [
  "whalecrab_engine"
]

# Wrapper for cargo test to reduce unnecessary rebuilds during tests
def --wrapped "cargo test" [...args: string] {
  let is_canary = "--profile canary" in ($args | str join " ")
  let crates = if $is_canary {
    $all_crates_with_canary_tests
  } else {
    $all_crates_with_tests
  }
  let crates_prefixed = $crates | each --flatten {
    ["--package", $in]
  }
  let final_args = $crates_prefixed ++ $args
  ^cargo test ...$final_args
}

# Run canary tests
def "cargo test canary" [] {
  cargo test --profile canary canary_ -- --ignored
}

# Run the full test suite, including canary tests
def "cargo test all" [] {
  cargo test --profile canary -- --include-ignored
}
