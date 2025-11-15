//! Example Blang Component using the Runtime
//!
//! This demonstrates how to build a simple counter component using the
//! blang_runtime API. In a real Blang application, this would be generated
//! from Blang source code.
//!
//! To compile this example to WASM:
//! ```bash
//! cargo build --target wasm32-unknown-unknown --release
//! ```

#![no_std]
#![no_main]

use blang_runtime::console;
use blang_runtime::dom::{self, ElementId};

/// Global state for the counter
static mut COUNTER_VALUE: i32 = 0;
static mut COUNTER_DISPLAY: ElementId = 0;

/// Panic handler for no_std
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Initialize the counter component
///
/// This function is called by the JS runtime when mounting the component.
/// It receives the container element ID and builds the UI inside it.
///
/// # Arguments
///
/// * `container_id` - Element ID of the container to render into
#[no_mangle]
pub extern "C" fn init_counter(container_id: ElementId) {
    console::log("Initializing counter component from WASM");

    // Create the counter display
    let counter_div = dom::create_element("div");
    dom::set_attribute(counter_div, "class", "counter");
    dom::set_text_content(counter_div, "0");

    unsafe {
        COUNTER_DISPLAY = counter_div;
    }

    // Create controls container
    let controls = dom::create_element("div");
    dom::set_attribute(controls, "class", "controls");

    // Create decrement button
    let dec_btn = dom::create_element("button");
    dom::set_text_content(dec_btn, "➖ Decrement");
    dom::add_event_listener(dec_btn, "click", 0); // Callback index 0

    // Create reset button
    let reset_btn = dom::create_element("button");
    dom::set_text_content(reset_btn, "🔄 Reset");
    dom::add_event_listener(reset_btn, "click", 1); // Callback index 1

    // Create increment button
    let inc_btn = dom::create_element("button");
    dom::set_text_content(inc_btn, "➕ Increment");
    dom::add_event_listener(inc_btn, "click", 2); // Callback index 2

    // Append buttons to controls
    dom::append_child(controls, dec_btn);
    dom::append_child(controls, reset_btn);
    dom::append_child(controls, inc_btn);

    // Append everything to container
    dom::append_child(container_id, counter_div);
    dom::append_child(container_id, controls);

    console::log("Counter component initialized");
}

/// Event callback for decrement button (index 0)
#[no_mangle]
pub extern "C" fn __event_callback_0(_elem_id: ElementId) {
    unsafe {
        COUNTER_VALUE -= 1;
        update_display();
    }
    console::log("Decremented");
}

/// Event callback for reset button (index 1)
#[no_mangle]
pub extern "C" fn __event_callback_1(_elem_id: ElementId) {
    unsafe {
        COUNTER_VALUE = 0;
        update_display();
    }
    console::log("Reset counter");
}

/// Event callback for increment button (index 2)
#[no_mangle]
pub extern "C" fn __event_callback_2(_elem_id: ElementId) {
    unsafe {
        COUNTER_VALUE += 1;
        update_display();
    }
    console::log("Incremented");
}

/// Update the counter display with current value
fn update_display() {
    unsafe {
        // Convert counter value to string
        let value = COUNTER_VALUE;
        let mut buffer = [0u8; 12];
        let s = format_i32(value, &mut buffer);

        dom::set_text_content(COUNTER_DISPLAY, s);
    }
}

/// Simple i32 to string formatter (since we're in no_std)
///
/// Returns a string slice representing the number.
fn format_i32(mut n: i32, buffer: &mut [u8]) -> &str {
    let is_negative = n < 0;
    if is_negative {
        n = -n;
    }

    let mut i = buffer.len();
    loop {
        i -= 1;
        buffer[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }

    if is_negative {
        i -= 1;
        buffer[i] = b'-';
    }

    unsafe { core::str::from_utf8_unchecked(&buffer[i..]) }
}

// Export memory so JS can read strings from it
#[no_mangle]
pub static mut __heap_base: u32 = 0;
