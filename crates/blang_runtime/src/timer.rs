//! Timer primitives for WASM-JS interop
//!
//! This module provides setTimeout/clearTimeout-like functionality for WebAssembly.
//! The actual timer management is handled by JavaScript using the browser's timer APIs.
//!
//! # Contract
//!
//! - **WASM side**: Registers callback function indices and receives timer IDs
//! - **JS side**: Uses setTimeout/clearTimeout and invokes WASM callbacks
//!
//! # Usage
//!
//! ```rust,no_run
//! use blang_runtime::timer;
//!
//! // Schedule a callback after 1000ms
//! // Assuming callback index 0 points to your callback function
//! let timer_id = timer::set_timeout(0, 1000);
//!
//! // Cancel if needed
//! timer::clear_timeout(timer_id);
//! ```
//!
//! # Callback Mechanism
//!
//! Since WASM can't directly pass function pointers to JS, we use a table-based approach:
//! 1. WASM exports a callback table or specific callback functions
//! 2. WASM passes a callback index to set_timeout
//! 3. JS stores the index with the timer
//! 4. When the timer fires, JS calls the WASM callback by index

/// Timer ID type returned by set_timeout
///
/// This ID can be used to cancel the timer with clear_timeout.
pub type TimerId = u32;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Import from JS: setTimeout
    ///
    /// Schedules a callback to be invoked after a delay.
    ///
    /// # Arguments
    ///
    /// * `callback_index` - Index of the callback function in WASM's callback table
    /// * `delay_ms` - Delay in milliseconds
    ///
    /// # Returns
    ///
    /// Timer ID that can be used with clear_timeout
    fn js_set_timeout(callback_index: u32, delay_ms: u32) -> TimerId;

    /// Import from JS: clearTimeout
    ///
    /// Cancels a previously scheduled timer.
    ///
    /// # Arguments
    ///
    /// * `timer_id` - Timer ID returned by set_timeout
    fn js_clear_timeout(timer_id: TimerId);

    /// Import from JS: setInterval
    ///
    /// Schedules a callback to be invoked repeatedly at an interval.
    ///
    /// # Arguments
    ///
    /// * `callback_index` - Index of the callback function
    /// * `interval_ms` - Interval in milliseconds
    ///
    /// # Returns
    ///
    /// Timer ID that can be used with clear_interval
    fn js_set_interval(callback_index: u32, interval_ms: u32) -> TimerId;

    /// Import from JS: clearInterval
    ///
    /// Cancels a previously scheduled interval.
    ///
    /// # Arguments
    ///
    /// * `timer_id` - Timer ID returned by set_interval
    fn js_clear_interval(timer_id: TimerId);
}

/// Schedule a callback to run after a delay
///
/// # Arguments
///
/// * `callback_index` - Index of the callback in your callback table
/// * `delay_ms` - Delay in milliseconds
///
/// # Returns
///
/// Timer ID that can be used to cancel the timer
///
/// # Example
///
/// ```rust,no_run
/// use blang_runtime::timer;
///
/// // Assuming callback index 0 points to your function
/// let timer_id = timer::set_timeout(0, 1000);
/// ```
#[cfg(target_arch = "wasm32")]
pub fn set_timeout(callback_index: u32, delay_ms: u32) -> TimerId {
    unsafe { js_set_timeout(callback_index, delay_ms) }
}

/// Cancel a scheduled timeout
///
/// # Arguments
///
/// * `timer_id` - Timer ID returned by set_timeout
///
/// # Example
///
/// ```rust,no_run
/// use blang_runtime::timer;
///
/// let timer_id = timer::set_timeout(0, 1000);
/// timer::clear_timeout(timer_id);
/// ```
#[cfg(target_arch = "wasm32")]
pub fn clear_timeout(timer_id: TimerId) {
    unsafe { js_clear_timeout(timer_id) }
}

/// Schedule a callback to run repeatedly at an interval
///
/// # Arguments
///
/// * `callback_index` - Index of the callback in your callback table
/// * `interval_ms` - Interval in milliseconds
///
/// # Returns
///
/// Timer ID that can be used to cancel the interval
///
/// # Example
///
/// ```rust,no_run
/// use blang_runtime::timer;
///
/// // Tick every second
/// let timer_id = timer::set_interval(0, 1000);
/// ```
#[cfg(target_arch = "wasm32")]
pub fn set_interval(callback_index: u32, interval_ms: u32) -> TimerId {
    unsafe { js_set_interval(callback_index, interval_ms) }
}

/// Cancel a scheduled interval
///
/// # Arguments
///
/// * `timer_id` - Timer ID returned by set_interval
///
/// # Example
///
/// ```rust,no_run
/// use blang_runtime::timer;
///
/// let timer_id = timer::set_interval(0, 1000);
/// // Stop after some time
/// timer::clear_interval(timer_id);
/// ```
#[cfg(target_arch = "wasm32")]
pub fn clear_interval(timer_id: TimerId) {
    unsafe { js_clear_interval(timer_id) }
}

// Non-WASM stubs for testing
#[cfg(not(target_arch = "wasm32"))]
pub fn set_timeout(_callback_index: u32, _delay_ms: u32) -> TimerId {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_timeout(_timer_id: TimerId) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_interval(_callback_index: u32, _interval_ms: u32) -> TimerId {
    0
}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_interval(_timer_id: TimerId) {}
