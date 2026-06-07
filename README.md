# llm-message-dispatch

[![CI](https://github.com/MukundaKatta/llm-message-dispatch-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/MukundaKatta/llm-message-dispatch-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](#license)

Route incoming messages to **named handlers** based on simple keyword or prefix
rules. Register a set of rules once, then ask the dispatcher which handler a
message belongs to.

This crate stores routing rules only — it does **not** run handler logic. It
returns the *name* of the matching handler so your application stays in control
of what actually happens. That makes it a tiny, dependency-free building block
for chat command routers, LLM agent tool selectors, webhook fan-out, and
similar dispatch problems.

- Zero dependencies, `#![forbid(unsafe_code)]`.
- Deterministic, priority-ordered routing.
- First-match (`dispatch`) and all-matches (`dispatch_all`) APIs.

## Installation

Add it to your `Cargo.toml`:

```toml
[dependencies]
llm-message-dispatch = "0.1"
```

Or with the Cargo CLI:

```sh
cargo add llm-message-dispatch
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

### Priorities

Routes are evaluated from highest priority to lowest. Use
`add_handler_with_priority` when ordering matters; `add_handler` defaults to
priority `0`. Among equal priorities, the first-registered handler wins.

```rust
use llm_message_dispatch::{Dispatcher, MatchRule};

let mut d = Dispatcher::new();
d.add_handler_with_priority("admin", MatchRule::prefix("/admin"), 100);
d.add_handler("search", MatchRule::contains("search"));
d.add_handler_with_priority("fallback", MatchRule::always(), -100);

// "/admin search logs" matches both the admin prefix and the "search"
// keyword, but the admin route has higher priority.
assert_eq!(d.dispatch("/admin search logs"), Some("admin"));
```

### Matching every handler

`dispatch_all` returns *all* matching handler names in priority order, which is
handy for fan-out (e.g. notifying several subscribers):

```rust
use llm_message_dispatch::{Dispatcher, MatchRule};

let mut d = Dispatcher::new();
d.add_handler_with_priority("search", MatchRule::contains("search"), 10);
d.add_handler("logger", MatchRule::always());

assert_eq!(d.dispatch_all("search the web"), vec!["search", "logger"]);
```

## Match rules

| Rule       | Constructor              | Matches when…                  | Case sensitivity |
|------------|--------------------------|--------------------------------|------------------|
| `Prefix`   | `MatchRule::prefix(s)`   | message starts with `s`        | case-sensitive   |
| `Contains` | `MatchRule::contains(s)` | message contains `s`           | case-insensitive |
| `Exact`    | `MatchRule::exact(s)`    | message equals `s`             | case-sensitive   |
| `Always`   | `MatchRule::always()`    | always (fallback handler)      | n/a              |

## API overview

### `Dispatcher`

| Method | Description |
|--------|-------------|
| `new()` | Create an empty dispatcher. |
| `add_handler(name, rule)` | Register a handler at priority `0`. |
| `add_handler_with_priority(name, rule, priority)` | Register a handler with an explicit priority. |
| `dispatch(message) -> Option<&str>` | First matching handler name (highest priority first). |
| `dispatch_all(message) -> Vec<&str>` | All matching handler names, in priority order. |
| `handler_count() -> usize` | Number of registered routes. |
| `is_empty() -> bool` | `true` when no routes are registered. |
| `contains_handler(name) -> bool` | Whether any route uses `name`. |
| `handlers() -> impl Iterator<Item = &str>` | Registered handler names in priority order. |
| `remove_handler(name) -> usize` | Remove all routes for `name`; returns how many were removed. |
| `clear()` | Remove every route. |

### `MatchRule`

An enum with variants `Prefix`, `Contains`, `Exact`, and `Always`, plus the
constructor helpers above and a `matches(&self, message: &str) -> bool` method.

## Building and testing

```sh
cargo build
cargo test          # unit tests, integration tests, and doc tests
cargo fmt --check   # formatting
cargo clippy --all-targets -- -D warnings
```

## License

Licensed under the [MIT License](https://opensource.org/licenses/MIT).
