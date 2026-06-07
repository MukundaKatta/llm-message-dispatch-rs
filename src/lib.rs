/*!
`llm-message-dispatch` — route messages to named handlers by keyword rules.

Register named handlers with keyword or prefix matching rules. When a
message arrives, [`Dispatcher::dispatch`] returns the first matching handler
name. This crate stores routes only — actual handler logic is bring-your-own.

# Quick start

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

# Match rules

| Rule | Constructor | Matches when… | Case sensitivity |
|------|-------------|---------------|------------------|
| Prefix   | [`MatchRule::prefix`]   | message starts with the prefix | sensitive |
| Contains | [`MatchRule::contains`] | message contains the keyword   | insensitive |
| Exact    | [`MatchRule::exact`]    | message equals the string      | sensitive |
| Always   | [`MatchRule::always`]   | always (fallback handler)      | n/a |

# Priority and ordering

Routes are evaluated from highest priority to lowest. Handlers added with
[`Dispatcher::add_handler`] default to priority `0`; use
[`Dispatcher::add_handler_with_priority`] to override. Among handlers with the
same priority, the one registered first wins (registration order is stable).

```rust
use llm_message_dispatch::{Dispatcher, MatchRule};

let mut d = Dispatcher::new();
d.add_handler_with_priority("fallback", MatchRule::always(), 0);
d.add_handler_with_priority("urgent", MatchRule::contains("urgent"), 10);

// "urgent" outranks the always-matching fallback.
assert_eq!(d.dispatch("urgent: server down"), Some("urgent"));
// Non-urgent messages still hit the fallback.
assert_eq!(d.dispatch("just saying hi"), Some("fallback"));
```
*/

#![forbid(unsafe_code)]

/// A match rule for message routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchRule {
    /// Matches if the message starts with the given prefix (case-sensitive).
    Prefix(String),
    /// Matches if the message contains the keyword (case-insensitive).
    Contains(String),
    /// Matches if the message equals the given string exactly (case-sensitive).
    Exact(String),
    /// Always matches (fallback/default handler).
    Always,
}

impl MatchRule {
    /// Build a [`MatchRule::Prefix`] rule.
    pub fn prefix(s: impl Into<String>) -> Self {
        Self::Prefix(s.into())
    }

    /// Build a [`MatchRule::Contains`] rule.
    pub fn contains(s: impl Into<String>) -> Self {
        Self::Contains(s.into())
    }

    /// Build a [`MatchRule::Exact`] rule.
    pub fn exact(s: impl Into<String>) -> Self {
        Self::Exact(s.into())
    }

    /// Build a [`MatchRule::Always`] rule.
    pub fn always() -> Self {
        Self::Always
    }

    /// Returns `true` if `message` satisfies this rule.
    ///
    /// ```
    /// use llm_message_dispatch::MatchRule;
    ///
    /// assert!(MatchRule::contains("RUST").matches("I love rust"));
    /// assert!(MatchRule::prefix("/").matches("/help"));
    /// assert!(!MatchRule::exact("hello").matches("hello world"));
    /// ```
    pub fn matches(&self, message: &str) -> bool {
        match self {
            MatchRule::Prefix(p) => message.starts_with(p.as_str()),
            MatchRule::Contains(k) => message.to_lowercase().contains(&k.to_lowercase()),
            MatchRule::Exact(e) => message == e.as_str(),
            MatchRule::Always => true,
        }
    }
}

#[derive(Debug, Clone)]
struct Route {
    handler: String,
    rule: MatchRule,
    priority: i32,
}

/// Routes messages to named handlers.
///
/// Routes are kept sorted by descending priority as they are inserted, so
/// [`dispatch`](Dispatcher::dispatch) and
/// [`dispatch_all`](Dispatcher::dispatch_all) run in a single pass without
/// re-sorting or allocating.
#[derive(Debug, Default, Clone)]
pub struct Dispatcher {
    /// Invariant: always sorted by descending `priority`, with insertion order
    /// preserved among equal priorities.
    routes: Vec<Route>,
}

impl Dispatcher {
    /// Create an empty dispatcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a handler with a match rule at the default priority (`0`).
    ///
    /// Among handlers of equal priority, the first registered wins.
    pub fn add_handler(&mut self, handler: impl Into<String>, rule: MatchRule) {
        self.add_handler_with_priority(handler, rule, 0);
    }

    /// Add a handler with a match rule and an explicit priority.
    ///
    /// Higher-priority routes are evaluated first. The route is inserted in
    /// sorted position, keeping insertion order stable among equal priorities.
    pub fn add_handler_with_priority(
        &mut self,
        handler: impl Into<String>,
        rule: MatchRule,
        priority: i32,
    ) {
        let route = Route {
            handler: handler.into(),
            rule,
            priority,
        };
        // Insert before the first route with strictly lower priority. Because
        // we skip equal priorities, earlier registrations stay ahead of later
        // ones at the same priority (stable, first-registered-wins ordering).
        let pos = self
            .routes
            .iter()
            .position(|r| r.priority < route.priority)
            .unwrap_or(self.routes.len());
        self.routes.insert(pos, route);
    }

    /// Dispatch a message; returns the first matching handler name.
    ///
    /// Routes with higher priority are checked first.
    pub fn dispatch<'a>(&'a self, message: &str) -> Option<&'a str> {
        self.routes
            .iter()
            .find(|r| r.rule.matches(message))
            .map(|r| r.handler.as_str())
    }

    /// All handler names that match `message`, in priority order.
    pub fn dispatch_all<'a>(&'a self, message: &str) -> Vec<&'a str> {
        self.routes
            .iter()
            .filter(|r| r.rule.matches(message))
            .map(|r| r.handler.as_str())
            .collect()
    }

    /// Number of registered routes.
    pub fn handler_count(&self) -> usize {
        self.routes.len()
    }

    /// Returns `true` if no routes are registered.
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    /// Returns `true` if at least one route uses the given handler name.
    pub fn contains_handler(&self, handler: &str) -> bool {
        self.routes.iter().any(|r| r.handler == handler)
    }

    /// Iterate over registered handler names in priority order (may repeat if
    /// a handler is registered under multiple rules).
    pub fn handlers(&self) -> impl Iterator<Item = &str> {
        self.routes.iter().map(|r| r.handler.as_str())
    }

    /// Remove all routes for a handler name. Returns the number removed.
    pub fn remove_handler(&mut self, handler: &str) -> usize {
        let before = self.routes.len();
        self.routes.retain(|r| r.handler != handler);
        before - self.routes.len()
    }

    /// Remove every route, leaving an empty dispatcher.
    pub fn clear(&mut self) {
        self.routes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_match() {
        let mut d = Dispatcher::new();
        d.add_handler("cmd", MatchRule::prefix("/"));
        assert_eq!(d.dispatch("/help"), Some("cmd"));
    }

    #[test]
    fn contains_match() {
        let mut d = Dispatcher::new();
        d.add_handler("search", MatchRule::contains("search"));
        assert_eq!(d.dispatch("please SEARCH for rust"), Some("search"));
    }

    #[test]
    fn exact_match() {
        let mut d = Dispatcher::new();
        d.add_handler("hi", MatchRule::exact("hello"));
        assert_eq!(d.dispatch("hello"), Some("hi"));
        assert_eq!(d.dispatch("hello world"), None);
    }

    #[test]
    fn always_matches() {
        let mut d = Dispatcher::new();
        d.add_handler("fallback", MatchRule::always());
        assert_eq!(d.dispatch("anything"), Some("fallback"));
    }

    #[test]
    fn first_registered_wins() {
        let mut d = Dispatcher::new();
        d.add_handler("first", MatchRule::contains("rust"));
        d.add_handler("second", MatchRule::contains("rust"));
        assert_eq!(d.dispatch("I love rust"), Some("first"));
    }

    #[test]
    fn no_match_returns_none() {
        let d = Dispatcher::new();
        assert_eq!(d.dispatch("hello"), None);
    }

    #[test]
    fn dispatch_all_returns_multiple() {
        let mut d = Dispatcher::new();
        d.add_handler("a", MatchRule::contains("rust"));
        d.add_handler("b", MatchRule::always());
        let matches = d.dispatch_all("rust is great");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn dispatch_all_in_priority_order() {
        let mut d = Dispatcher::new();
        d.add_handler_with_priority("low", MatchRule::always(), 0);
        d.add_handler_with_priority("high", MatchRule::always(), 10);
        d.add_handler_with_priority("mid", MatchRule::always(), 5);
        assert_eq!(d.dispatch_all("x"), vec!["high", "mid", "low"]);
    }

    #[test]
    fn priority_higher_wins() {
        let mut d = Dispatcher::new();
        d.add_handler_with_priority("low", MatchRule::always(), 0);
        d.add_handler_with_priority("high", MatchRule::always(), 10);
        assert_eq!(d.dispatch("x"), Some("high"));
    }

    #[test]
    fn priority_higher_wins_regardless_of_insertion_order() {
        // Add the high-priority route *after* the low one; it must still win.
        let mut d = Dispatcher::new();
        d.add_handler_with_priority("high", MatchRule::always(), 10);
        d.add_handler_with_priority("low", MatchRule::always(), 0);
        assert_eq!(d.dispatch("x"), Some("high"));
    }

    #[test]
    fn equal_priority_keeps_registration_order() {
        let mut d = Dispatcher::new();
        d.add_handler_with_priority("first", MatchRule::always(), 5);
        d.add_handler_with_priority("second", MatchRule::always(), 5);
        assert_eq!(d.dispatch_all("x"), vec!["first", "second"]);
    }

    #[test]
    fn negative_priority_ranks_below_default() {
        let mut d = Dispatcher::new();
        d.add_handler("default", MatchRule::always());
        d.add_handler_with_priority("last_resort", MatchRule::always(), -10);
        assert_eq!(d.dispatch_all("x"), vec!["default", "last_resort"]);
    }

    #[test]
    fn remove_handler_reports_count_and_clears_route() {
        let mut d = Dispatcher::new();
        d.add_handler("gone", MatchRule::always());
        d.add_handler("gone", MatchRule::contains("x"));
        assert_eq!(d.remove_handler("gone"), 2);
        assert_eq!(d.remove_handler("missing"), 0);
        assert_eq!(d.dispatch("x"), None);
    }

    #[test]
    fn handler_count() {
        let mut d = Dispatcher::new();
        d.add_handler("a", MatchRule::always());
        d.add_handler("b", MatchRule::always());
        assert_eq!(d.handler_count(), 2);
    }

    #[test]
    fn is_empty_and_clear() {
        let mut d = Dispatcher::new();
        assert!(d.is_empty());
        d.add_handler("a", MatchRule::always());
        assert!(!d.is_empty());
        d.clear();
        assert!(d.is_empty());
        assert_eq!(d.dispatch("x"), None);
    }

    #[test]
    fn contains_handler_and_handlers_iter() {
        let mut d = Dispatcher::new();
        d.add_handler_with_priority("b", MatchRule::always(), 1);
        d.add_handler_with_priority("a", MatchRule::always(), 2);
        assert!(d.contains_handler("a"));
        assert!(!d.contains_handler("z"));
        // Highest priority first.
        assert_eq!(d.handlers().collect::<Vec<_>>(), vec!["a", "b"]);
    }

    #[test]
    fn prefix_no_match() {
        let mut d = Dispatcher::new();
        d.add_handler("cmd", MatchRule::prefix("/"));
        assert_eq!(d.dispatch("no slash"), None);
    }

    #[test]
    fn case_insensitive_contains() {
        let mut d = Dispatcher::new();
        d.add_handler("greet", MatchRule::contains("HELLO"));
        assert_eq!(d.dispatch("hello there"), Some("greet"));
    }

    #[test]
    fn prefix_is_case_sensitive() {
        let mut d = Dispatcher::new();
        d.add_handler("cmd", MatchRule::prefix("/Help"));
        assert_eq!(d.dispatch("/Help me"), Some("cmd"));
        assert_eq!(d.dispatch("/help me"), None);
    }

    #[test]
    fn empty_message_only_hits_always_and_empty_rules() {
        let mut d = Dispatcher::new();
        d.add_handler("fallback", MatchRule::always());
        d.add_handler("blank", MatchRule::exact(""));
        d.add_handler("any_prefix", MatchRule::prefix(""));
        let matches = d.dispatch_all("");
        assert!(matches.contains(&"fallback"));
        assert!(matches.contains(&"blank"));
        assert!(matches.contains(&"any_prefix"));
    }

    #[test]
    fn match_rule_matches_directly() {
        assert!(MatchRule::always().matches(""));
        assert!(MatchRule::contains("ab").matches("xxABxx"));
        assert!(!MatchRule::exact("ab").matches("abc"));
    }
}
