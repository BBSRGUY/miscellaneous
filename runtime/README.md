# Blang WebAssembly Runtime

A complete runtime system that bridges Blang WebAssembly modules with browser JavaScript environments.

## Overview

The Blang runtime consists of two parts:

1. **Rust Side** (`blang_runtime` crate): Defines the FFI interface and provides utilities for WASM modules
2. **JavaScript Side** (`runtime.js`): Implements host functionality using browser APIs

## Architecture

```
┌─────────────────────────────────────┐
│   Blang Source Code (.blang)       │
└──────────────┬──────────────────────┘
               │ Compile
               ▼
┌─────────────────────────────────────┐
│   Blang IR + Type System            │
└──────────────┬──────────────────────┘
               │ Codegen
               ▼
┌─────────────────────────────────────┐
│   WebAssembly Module (.wasm)        │
│   Uses: blang_runtime API           │
└──────────────┬──────────────────────┘
               │ Load & Instantiate
               ▼
┌─────────────────────────────────────┐
│   runtime.js (Browser)              │
│   Provides: DOM, Console, Timers    │
└──────────────┬──────────────────────┘
               │ Render
               ▼
┌─────────────────────────────────────┐
│   Browser DOM                       │
└─────────────────────────────────────┘
```

## Features

### 🎯 DOM Manipulation
- Create, modify, and remove DOM elements
- No `innerHTML` string hacking - uses proper DOM APIs
- Element ID system for safe WASM-JS communication
- Full support for attributes, properties, and styles

### 📝 Console Operations
- `console.log`, `console.error`, `console.warn`
- UTF-8 string encoding/decoding
- Visible in browser dev tools

### ⏱️ Timers
- `setTimeout` and `setInterval`
- Callback support via function tables
- Proper cleanup with `clearTimeout`/`clearInterval`

### 💾 Memory Management
- Simple bump allocator for WASM
- UTF-8 string allocation helpers
- Memory is managed by WASM, JS only reads

### 🔄 Reactive State
- Component state management
- Change detection and handlers
- Automatic re-rendering

## Usage

### Rust Side (WASM Component)

```rust
use blang_runtime::{console, dom};

#[no_mangle]
pub extern "C" fn init_my_component(container_id: dom::ElementId) {
    console::log("Hello from WASM!");

    // Create UI elements
    let button = dom::create_element("button");
    dom::set_text_content(button, "Click me!");
    dom::add_event_listener(button, "click", 0);

    // Mount to container
    dom::append_child(container_id, button);
}

#[no_mangle]
pub extern "C" fn __event_callback_0(elem_id: dom::ElementId) {
    console::log("Button clicked!");
}
```

### JavaScript Side (Browser)

```javascript
import { loadWasmModule, mountComponent } from './runtime.js';

async function main() {
    // Load the WASM module
    const response = await fetch('component.wasm');
    const wasmBytes = await response.arrayBuffer();
    const instance = await loadWasmModule(wasmBytes);

    // Mount component to DOM
    mountComponent('#app', 'my_component', instance);
}

main();
```

### HTML

```html
<!DOCTYPE html>
<html>
<head>
    <title>My Blang App</title>
</head>
<body>
    <div id="app"></div>
    <script type="module" src="main.js"></script>
</body>
</html>
```

## API Reference

### DOM Functions

| Function | Description |
|----------|-------------|
| `create_element(tag)` | Create a new DOM element |
| `set_attribute(id, key, val)` | Set an attribute |
| `set_text_content(id, text)` | Set text content |
| `append_child(parent, child)` | Append child to parent |
| `add_event_listener(id, event, callback)` | Add event listener |
| `query_selector(selector)` | Find element by selector |
| `remove_element(id)` | Remove element from DOM |
| `set_property(id, prop, val)` | Set element property |
| `set_style(id, prop, val)` | Set CSS style |

### Console Functions

| Function | Description |
|----------|-------------|
| `console::log(msg)` | Log message |
| `console::error(msg)` | Log error |
| `console::warn(msg)` | Log warning |

### Timer Functions

| Function | Description |
|----------|-------------|
| `set_timeout(callback, delay)` | Schedule one-time callback |
| `clear_timeout(id)` | Cancel timeout |
| `set_interval(callback, interval)` | Schedule repeating callback |
| `clear_interval(id)` | Cancel interval |

## Contract Details

### String Encoding

Strings are passed between WASM and JS as `(pointer, length)` pairs:
- **WASM → JS**: Pointer to UTF-8 bytes in WASM linear memory + byte length
- **JS → WASM**: JS allocates in WASM memory if needed

Example:
```rust
let msg = "Hello";
let bytes = msg.as_bytes();
unsafe {
    console_log(bytes.as_ptr(), bytes.len() as u32);
}
```

JavaScript reads:
```javascript
function readString(memory, ptr, len) {
    const bytes = new Uint8Array(memory.buffer, ptr, len);
    return textDecoder.decode(bytes);
}
```

### Element IDs

DOM elements are identified by numeric IDs:
- **ID 0** is reserved (invalid/error)
- **ID 1+** are valid element IDs
- JavaScript maintains a `Map<number, HTMLElement>`
- WASM only works with numeric IDs

This prevents WASM from holding raw pointers to JS objects.

### Event Callbacks

Event listeners use a callback index system:
- WASM exports functions named `__event_callback_{index}`
- WASM passes callback index when adding listener
- JS calls the corresponding export when event fires

Example:
```rust
dom::add_event_listener(button_id, "click", 0);
// JS will call __event_callback_0(button_id) on click
```

### Timer Callbacks

Similar to events, timers use callback indices:
- WASM exports functions named `__timer_callback_{index}`
- When timer fires, JS calls the corresponding export

## Examples

See:
- `example.html` - Interactive browser example
- `example_component.rs` - Complete counter component in Rust
- `runtime.js` - Full runtime implementation

## Building Components

### Compile Rust to WASM

```bash
# From your component directory
cargo build --target wasm32-unknown-unknown --release

# WASM output will be in:
# target/wasm32-unknown-unknown/release/your_component.wasm
```

### Optimize WASM (Optional)

```bash
# Install wasm-opt
npm install -g wasm-opt

# Optimize
wasm-opt -Oz -o optimized.wasm original.wasm
```

### Serve and Test

```bash
# Simple HTTP server
python3 -m http.server 8000

# Or use any server that serves WASM with correct MIME type
```

## Security

- **No `innerHTML`**: All DOM manipulation uses safe APIs
- **No `eval`**: No dynamic code execution
- **Element ID isolation**: WASM can't access arbitrary DOM nodes
- **Memory safety**: WASM can't access JS heap, only its own linear memory
- **UTF-8 validation**: Strings are properly decoded/validated

## Performance

- **Element ID lookup**: O(1) via HashMap
- **String encoding**: Zero-copy when reading from WASM memory
- **Event callbacks**: Direct function calls, no serialization
- **Memory**: Bump allocator is very fast for temporary allocations

## Limitations

- **Bump allocator**: Memory is never freed (suitable for short-lived instances)
- **Single-threaded**: No Web Workers support yet
- **Synchronous**: No async/await in WASM callbacks
- **No GC coordination**: WASM and JS heaps are separate

## Future Enhancements

- [ ] Web Workers support for multi-threading
- [ ] Async callback support
- [ ] Canvas/WebGL bindings
- [ ] Fetch API integration
- [ ] LocalStorage/IndexedDB
- [ ] Better memory allocator (e.g., wee_alloc)

## License

See repository root for license information.
