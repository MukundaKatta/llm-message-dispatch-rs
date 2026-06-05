# llm-message-dispatch

Route incoming messages to named handlers based on keyword or prefix rules.

`llm-message-dispatch` is a small, dependency-free Rust library for building
message routers — for example, deciding which handler should process an
incoming chat or agent message. You register named handlers together with
match rules, and the dispatcher returns the name of the first (or all)
matching handler(s). The crate stores routes only; the actual handler logic
is bring-your-own.

## Features

- Multiple match rules: prefix, case-insensitive contains, exact, and an
  always-matching fallback.
- Priority-based dispatch — higher-priority routes are checked first.
- Return the first matching handler (`dispatch`) or every match
  (`dispatch_all`).
- Add and remove handlers at runtime.
- Zero runtime dependencies.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
llm-message-dispatch = "0.1"
```

## Usage

```rust
use llm_message_dispatch::{Dispatcher, MatchRule};

let mut d = Dispatcher::new();
d.add_handler("help", MatchRule::prefix("/help"));
d.add_handler("search", MatchRule::contains("search"));
d.add_handler("default", MatchRule::always());

assert_eq!(d.dispatch("/help me"), Some("help"));
assert_eq!(d.dispatch("please search for rust"), Some("search"));
assert_eq!(d.dispatch("anything else"), Some("default"));
```

### Match rules

| Rule                  | Matches when                                       |
| --------------------- | -------------------------------------------------- |
| `MatchRule::prefix`   | the message starts with the given prefix           |
| `MatchRule::contains` | the message contains the keyword (case-insensitive)|
| `MatchRule::exact`    | the message equals the given string exactly        |
| `MatchRule::always`   | always (use as a fallback / default handler)       |

### Priorities

By default every handler has priority `0` and is evaluated in registration
order, so the first-registered handler wins among equal matches. Use
`add_handler_with_priority` to make some routes win regardless of order —
higher priority is checked first.

```rust
use llm_message_dispatch::{Dispatcher, MatchRule};

let mut d = Dispatcher::new();
d.add_handler_with_priority("low", MatchRule::always(), 0);
d.add_handler_with_priority("high", MatchRule::always(), 10);

assert_eq!(d.dispatch("anything"), Some("high"));
```

### Matching all handlers

```rust
use llm_message_dispatch::{Dispatcher, MatchRule};

let mut d = Dispatcher::new();
d.add_handler("search", MatchRule::contains("rust"));
d.add_handler("fallback", MatchRule::always());

let matches = d.dispatch_all("rust is great");
assert_eq!(matches, vec!["search", "fallback"]);
```

## API overview

- `Dispatcher::new()` — create an empty dispatcher.
- `add_handler(name, rule)` — register a handler at default priority.
- `add_handler_with_priority(name, rule, priority)` — register with an
  explicit priority.
- `dispatch(message) -> Option<&str>` — first matching handler name.
- `dispatch_all(message) -> Vec<&str>` — all matching handler names, in
  priority order.
- `handler_count() -> usize` — number of registered routes.
- `remove_handler(name)` — remove all routes for a handler name.

## Building and testing

```sh
cargo build
cargo test
```

## Tech stack

- Rust (edition 2021)
- No external dependencies

## License

Licensed under the MIT License.
