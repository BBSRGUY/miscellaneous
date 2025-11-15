//! Integration tests for blang_runtime
//!
//! These tests verify that the runtime API works correctly in non-WASM environments.

use blang_runtime::*;

#[test]
fn test_console_log() {
    // Should not panic
    console::log("Test message");
    console::error("Test error");
    console::warn("Test warning");
}

#[test]
fn test_dom_operations() {
    // Create element should return a non-zero ID
    let elem_id = dom::create_element("div");
    assert_ne!(elem_id, 0);

    // Setting attributes should not panic
    dom::set_attribute(elem_id, "class", "test");
    dom::set_text_content(elem_id, "Hello");
}

#[test]
fn test_timer_operations() {
    // Timer operations should not panic
    let timer_id = timer::set_timeout(0, 100);
    timer::clear_timeout(timer_id);

    let interval_id = timer::set_interval(0, 100);
    timer::clear_interval(interval_id);
}

#[test]
fn test_memory_operations() {
    // Allocate should return null in non-WASM (stub implementation)
    let ptr = memory::alloc(100);
    assert!(ptr.is_null());

    // Dealloc should not panic
    memory::dealloc(ptr, 100);
}

#[test]
fn test_element_operations() {
    let parent = dom::create_element("div");
    let child = dom::create_element("span");

    dom::append_child(parent, child);
    dom::set_property(child, "value", "test");
    dom::set_style(child, "color", "red");
}

#[test]
fn test_query_selector() {
    let elem_id = dom::query_selector("#app");
    assert_ne!(elem_id, 0);
}

#[test]
fn test_event_listener() {
    let button = dom::create_element("button");
    dom::add_event_listener(button, "click", 0);
}
