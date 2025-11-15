//! DOM manipulation abstraction for WASM-JS interop
//!
//! This module provides a safe abstraction over browser DOM operations.
//! Elements are identified by opaque IDs rather than pointers, and all
//! actual DOM manipulation is performed by JavaScript.
//!
//! # Contract
//!
//! - **WASM side**: Calls these functions with element IDs and string data
//! - **JS side**: Maintains a map of element IDs to actual DOM nodes
//!
//! # Element ID System
//!
//! Each DOM element is assigned a unique numeric ID:
//! - ID 0 is reserved (invalid element)
//! - IDs are managed by JavaScript
//! - WASM only deals with opaque numeric identifiers
//!
//! # Usage
//!
//! ```rust,no_run
//! use blang_runtime::dom;
//!
//! // Create a div element
//! let div_id = dom::create_element("div");
//!
//! // Set attributes
//! dom::set_attribute(div_id, "class", "container");
//! dom::set_attribute(div_id, "id", "main");
//!
//! // Set text content
//! dom::set_text_content(div_id, "Hello, World!");
//!
//! // Create a button and append it
//! let button_id = dom::create_element("button");
//! dom::set_text_content(button_id, "Click me");
//! dom::append_child(div_id, button_id);
//!
//! // Add event listener (callback index 0)
//! dom::add_event_listener(button_id, "click", 0);
//!
//! // Mount to existing element
//! let container_id = dom::query_selector("#app");
//! dom::append_child(container_id, div_id);
//! ```

/// Opaque element ID type
///
/// This represents a DOM element in the JavaScript environment.
/// ID 0 is reserved and represents "no element" or an error.
pub type ElementId = u32;

/// Invalid element ID constant
pub const INVALID_ELEMENT: ElementId = 0;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Import from JS: createElement
    ///
    /// Creates a new DOM element with the specified tag name.
    ///
    /// # Arguments
    ///
    /// * `tag_ptr` - Pointer to UTF-8 encoded tag name
    /// * `tag_len` - Length of tag name in bytes
    ///
    /// # Returns
    ///
    /// Element ID, or 0 if creation failed
    fn dom_create_element(tag_ptr: *const u8, tag_len: u32) -> ElementId;

    /// Import from JS: setAttribute
    ///
    /// Sets an attribute on a DOM element.
    ///
    /// # Arguments
    ///
    /// * `elem_id` - Element ID
    /// * `key_ptr` - Pointer to attribute name
    /// * `key_len` - Length of attribute name
    /// * `val_ptr` - Pointer to attribute value
    /// * `val_len` - Length of attribute value
    fn dom_set_attribute(
        elem_id: ElementId,
        key_ptr: *const u8,
        key_len: u32,
        val_ptr: *const u8,
        val_len: u32,
    );

    /// Import from JS: setTextContent
    ///
    /// Sets the text content of a DOM element.
    ///
    /// # Arguments
    ///
    /// * `elem_id` - Element ID
    /// * `text_ptr` - Pointer to UTF-8 encoded text
    /// * `text_len` - Length of text in bytes
    fn dom_set_text_content(elem_id: ElementId, text_ptr: *const u8, text_len: u32);

    /// Import from JS: appendChild
    ///
    /// Appends a child element to a parent element.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - Parent element ID
    /// * `child_id` - Child element ID
    fn dom_append_child(parent_id: ElementId, child_id: ElementId);

    /// Import from JS: addEventListener
    ///
    /// Adds an event listener to a DOM element.
    ///
    /// # Arguments
    ///
    /// * `elem_id` - Element ID
    /// * `event_ptr` - Pointer to event name (e.g., "click")
    /// * `event_len` - Length of event name
    /// * `callback_index` - Index of callback function in WASM
    fn dom_add_event_listener(
        elem_id: ElementId,
        event_ptr: *const u8,
        event_len: u32,
        callback_index: u32,
    );

    /// Import from JS: querySelector
    ///
    /// Finds an existing DOM element by CSS selector.
    ///
    /// # Arguments
    ///
    /// * `selector_ptr` - Pointer to CSS selector string
    /// * `selector_len` - Length of selector string
    ///
    /// # Returns
    ///
    /// Element ID, or 0 if not found
    fn dom_query_selector(selector_ptr: *const u8, selector_len: u32) -> ElementId;

    /// Import from JS: removeElement
    ///
    /// Removes a DOM element from the document.
    ///
    /// # Arguments
    ///
    /// * `elem_id` - Element ID to remove
    fn dom_remove_element(elem_id: ElementId);

    /// Import from JS: removeChild
    ///
    /// Removes a child element from its parent.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - Parent element ID
    /// * `child_id` - Child element ID to remove
    fn dom_remove_child(parent_id: ElementId, child_id: ElementId);

    /// Import from JS: insertBefore
    ///
    /// Inserts a new child before an existing child.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - Parent element ID
    /// * `new_child_id` - New child to insert
    /// * `reference_id` - Existing child to insert before
    fn dom_insert_before(parent_id: ElementId, new_child_id: ElementId, reference_id: ElementId);

    /// Import from JS: setProperty
    ///
    /// Sets a property on a DOM element (e.g., element.value = "foo").
    ///
    /// # Arguments
    ///
    /// * `elem_id` - Element ID
    /// * `prop_ptr` - Pointer to property name
    /// * `prop_len` - Length of property name
    /// * `val_ptr` - Pointer to property value
    /// * `val_len` - Length of property value
    fn dom_set_property(
        elem_id: ElementId,
        prop_ptr: *const u8,
        prop_len: u32,
        val_ptr: *const u8,
        val_len: u32,
    );

    /// Import from JS: setStyle
    ///
    /// Sets a CSS style property on an element.
    ///
    /// # Arguments
    ///
    /// * `elem_id` - Element ID
    /// * `prop_ptr` - Pointer to style property name (e.g., "backgroundColor")
    /// * `prop_len` - Length of property name
    /// * `val_ptr` - Pointer to style value
    /// * `val_len` - Length of style value
    fn dom_set_style(
        elem_id: ElementId,
        prop_ptr: *const u8,
        prop_len: u32,
        val_ptr: *const u8,
        val_len: u32,
    );
}

/// Create a new DOM element
///
/// # Arguments
///
/// * `tag` - HTML tag name (e.g., "div", "button", "span")
///
/// # Returns
///
/// Element ID, or INVALID_ELEMENT if creation failed
#[cfg(target_arch = "wasm32")]
pub fn create_element(tag: &str) -> ElementId {
    let bytes = tag.as_bytes();
    unsafe { dom_create_element(bytes.as_ptr(), bytes.len() as u32) }
}

/// Set an attribute on a DOM element
///
/// # Arguments
///
/// * `elem_id` - Element ID
/// * `key` - Attribute name
/// * `value` - Attribute value
#[cfg(target_arch = "wasm32")]
pub fn set_attribute(elem_id: ElementId, key: &str, value: &str) {
    let key_bytes = key.as_bytes();
    let val_bytes = value.as_bytes();
    unsafe {
        dom_set_attribute(
            elem_id,
            key_bytes.as_ptr(),
            key_bytes.len() as u32,
            val_bytes.as_ptr(),
            val_bytes.len() as u32,
        )
    }
}

/// Set the text content of a DOM element
///
/// # Arguments
///
/// * `elem_id` - Element ID
/// * `text` - Text content
#[cfg(target_arch = "wasm32")]
pub fn set_text_content(elem_id: ElementId, text: &str) {
    let bytes = text.as_bytes();
    unsafe { dom_set_text_content(elem_id, bytes.as_ptr(), bytes.len() as u32) }
}

/// Append a child element to a parent
///
/// # Arguments
///
/// * `parent_id` - Parent element ID
/// * `child_id` - Child element ID
#[cfg(target_arch = "wasm32")]
pub fn append_child(parent_id: ElementId, child_id: ElementId) {
    unsafe { dom_append_child(parent_id, child_id) }
}

/// Add an event listener to a DOM element
///
/// # Arguments
///
/// * `elem_id` - Element ID
/// * `event` - Event name (e.g., "click", "input", "change")
/// * `callback_index` - Index of callback function in your callback table
#[cfg(target_arch = "wasm32")]
pub fn add_event_listener(elem_id: ElementId, event: &str, callback_index: u32) {
    let bytes = event.as_bytes();
    unsafe { dom_add_event_listener(elem_id, bytes.as_ptr(), bytes.len() as u32, callback_index) }
}

/// Find an existing DOM element by CSS selector
///
/// # Arguments
///
/// * `selector` - CSS selector (e.g., "#app", ".container")
///
/// # Returns
///
/// Element ID, or INVALID_ELEMENT if not found
#[cfg(target_arch = "wasm32")]
pub fn query_selector(selector: &str) -> ElementId {
    let bytes = selector.as_bytes();
    unsafe { dom_query_selector(bytes.as_ptr(), bytes.len() as u32) }
}

/// Remove a DOM element from the document
///
/// # Arguments
///
/// * `elem_id` - Element ID to remove
#[cfg(target_arch = "wasm32")]
pub fn remove_element(elem_id: ElementId) {
    unsafe { dom_remove_element(elem_id) }
}

/// Remove a child element from its parent
///
/// # Arguments
///
/// * `parent_id` - Parent element ID
/// * `child_id` - Child element ID to remove
#[cfg(target_arch = "wasm32")]
pub fn remove_child(parent_id: ElementId, child_id: ElementId) {
    unsafe { dom_remove_child(parent_id, child_id) }
}

/// Insert a new child before an existing child
///
/// # Arguments
///
/// * `parent_id` - Parent element ID
/// * `new_child_id` - New child to insert
/// * `reference_id` - Existing child to insert before
#[cfg(target_arch = "wasm32")]
pub fn insert_before(parent_id: ElementId, new_child_id: ElementId, reference_id: ElementId) {
    unsafe { dom_insert_before(parent_id, new_child_id, reference_id) }
}

/// Set a property on a DOM element
///
/// # Arguments
///
/// * `elem_id` - Element ID
/// * `property` - Property name (e.g., "value", "checked")
/// * `value` - Property value
#[cfg(target_arch = "wasm32")]
pub fn set_property(elem_id: ElementId, property: &str, value: &str) {
    let prop_bytes = property.as_bytes();
    let val_bytes = value.as_bytes();
    unsafe {
        dom_set_property(
            elem_id,
            prop_bytes.as_ptr(),
            prop_bytes.len() as u32,
            val_bytes.as_ptr(),
            val_bytes.len() as u32,
        )
    }
}

/// Set a CSS style property on an element
///
/// # Arguments
///
/// * `elem_id` - Element ID
/// * `property` - CSS property name (e.g., "backgroundColor", "fontSize")
/// * `value` - CSS value
#[cfg(target_arch = "wasm32")]
pub fn set_style(elem_id: ElementId, property: &str, value: &str) {
    let prop_bytes = property.as_bytes();
    let val_bytes = value.as_bytes();
    unsafe {
        dom_set_style(
            elem_id,
            prop_bytes.as_ptr(),
            prop_bytes.len() as u32,
            val_bytes.as_ptr(),
            val_bytes.len() as u32,
        )
    }
}

// Non-WASM stubs for testing
#[cfg(not(target_arch = "wasm32"))]
pub fn create_element(_tag: &str) -> ElementId {
    1
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_attribute(_elem_id: ElementId, _key: &str, _value: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_text_content(_elem_id: ElementId, _text: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn append_child(_parent_id: ElementId, _child_id: ElementId) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn add_event_listener(_elem_id: ElementId, _event: &str, _callback_index: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn query_selector(_selector: &str) -> ElementId {
    1
}

#[cfg(not(target_arch = "wasm32"))]
pub fn remove_element(_elem_id: ElementId) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn remove_child(_parent_id: ElementId, _child_id: ElementId) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn insert_before(_parent_id: ElementId, _new_child_id: ElementId, _reference_id: ElementId) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_property(_elem_id: ElementId, _property: &str, _value: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_style(_elem_id: ElementId, _property: &str, _value: &str) {}
