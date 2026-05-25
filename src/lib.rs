/*!
llm-message-dispatch: route messages to named handlers by keyword rules.

Register named handlers with keyword or prefix matching rules. When a
message arrives, `dispatch` returns the first matching handler name.
This crate stores routes only — actual handler logic is BYO.

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
*/

/// A match rule for message routing.
#[derive(Debug, Clone)]
pub enum MatchRule {
    /// Matches if message starts with the given prefix.
    Prefix(String),
    /// Matches if message contains the keyword (case-insensitive).
    Contains(String),
    /// Matches if message equals the given string exactly.
    Exact(String),
    /// Always matches (fallback/default handler).
    Always,
}

impl MatchRule {
    pub fn prefix(s: impl Into<String>) -> Self { Self::Prefix(s.into()) }
    pub fn contains(s: impl Into<String>) -> Self { Self::Contains(s.into()) }
    pub fn exact(s: impl Into<String>) -> Self { Self::Exact(s.into()) }
    pub fn always() -> Self { Self::Always }

    fn matches(&self, message: &str) -> bool {
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
#[derive(Debug, Default)]
pub struct Dispatcher {
    routes: Vec<Route>,
}

impl Dispatcher {
    pub fn new() -> Self { Self::default() }

    /// Add a handler with a match rule. First-registered wins among equals.
    pub fn add_handler(&mut self, handler: impl Into<String>, rule: MatchRule) {
        self.routes.push(Route { handler: handler.into(), rule, priority: 0 });
    }

    pub fn add_handler_with_priority(
        &mut self,
        handler: impl Into<String>,
        rule: MatchRule,
        priority: i32,
    ) {
        self.routes.push(Route { handler: handler.into(), rule, priority });
    }

    /// Dispatch a message; returns the first matching handler name.
    /// Routes with higher priority are checked first.
    pub fn dispatch<'a>(&'a self, message: &str) -> Option<&'a str> {
        let mut sorted: Vec<&Route> = self.routes.iter().collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        sorted.iter().find(|r| r.rule.matches(message)).map(|r| r.handler.as_str())
    }

    /// All handler names that match (in priority order).
    pub fn dispatch_all<'a>(&'a self, message: &str) -> Vec<&'a str> {
        let mut sorted: Vec<&Route> = self.routes.iter().collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        sorted.iter().filter(|r| r.rule.matches(message)).map(|r| r.handler.as_str()).collect()
    }

    pub fn handler_count(&self) -> usize { self.routes.len() }

    /// Remove all routes for a handler name.
    pub fn remove_handler(&mut self, handler: &str) {
        self.routes.retain(|r| r.handler != handler);
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
    fn priority_higher_wins() {
        let mut d = Dispatcher::new();
        d.add_handler_with_priority("low", MatchRule::always(), 0);
        d.add_handler_with_priority("high", MatchRule::always(), 10);
        assert_eq!(d.dispatch("x"), Some("high"));
    }

    #[test]
    fn remove_handler() {
        let mut d = Dispatcher::new();
        d.add_handler("gone", MatchRule::always());
        d.remove_handler("gone");
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
}
