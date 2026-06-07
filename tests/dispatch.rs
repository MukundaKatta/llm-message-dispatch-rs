//! Integration tests exercising the public API the way a downstream crate would.

use llm_message_dispatch::{Dispatcher, MatchRule};

/// A small realistic routing table similar to a chat command router.
fn build_router() -> Dispatcher {
    let mut d = Dispatcher::new();
    d.add_handler_with_priority("admin", MatchRule::prefix("/admin"), 100);
    d.add_handler("help", MatchRule::prefix("/help"));
    d.add_handler("search", MatchRule::contains("search"));
    d.add_handler("greeting", MatchRule::exact("hello"));
    d.add_handler_with_priority("fallback", MatchRule::always(), -100);
    d
}

#[test]
fn routes_commands_to_expected_handlers() {
    let d = build_router();
    assert_eq!(d.dispatch("/admin ban user"), Some("admin"));
    assert_eq!(d.dispatch("/help"), Some("help"));
    assert_eq!(d.dispatch("can you search the docs"), Some("search"));
    assert_eq!(d.dispatch("hello"), Some("greeting"));
    assert_eq!(d.dispatch("random chatter"), Some("fallback"));
}

#[test]
fn high_priority_prefix_beats_lower_priority_rules() {
    let d = build_router();
    // "/admin search" contains "search" but the admin prefix has higher priority.
    assert_eq!(d.dispatch("/admin search logs"), Some("admin"));
}

#[test]
fn dispatch_all_collects_every_match_in_priority_order() {
    let d = build_router();
    // "/admin search" matches admin (prefix), search (contains) and fallback.
    let all = d.dispatch_all("/admin search logs");
    assert_eq!(all, vec!["admin", "search", "fallback"]);
}

#[test]
fn fallback_only_when_nothing_else_matches() {
    let d = build_router();
    let all = d.dispatch_all("plain text");
    assert_eq!(all, vec!["fallback"]);
}

#[test]
fn removing_a_handler_changes_routing() {
    let mut d = build_router();
    assert_eq!(d.remove_handler("search"), 1);
    // Without the search route, the message now falls through to the fallback.
    assert_eq!(d.dispatch("please search"), Some("fallback"));
}

#[test]
fn clear_empties_the_router() {
    let mut d = build_router();
    d.clear();
    assert!(d.is_empty());
    assert_eq!(d.dispatch("/admin"), None);
}

#[test]
fn empty_dispatcher_returns_none() {
    let d = Dispatcher::new();
    assert!(d.is_empty());
    assert_eq!(d.dispatch("anything"), None);
    assert!(d.dispatch_all("anything").is_empty());
}
