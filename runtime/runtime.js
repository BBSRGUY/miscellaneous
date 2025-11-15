/**
 * Blang WebAssembly Runtime
 *
 * This module provides the JavaScript side of the Blang WASM runtime.
 * It handles:
 * - Loading and instantiating WASM modules
 * - Providing host implementations for console, timer, and DOM operations
 * - Managing element IDs and event listeners
 * - Component mounting and reactive updates
 *
 * Contract between WASM and JS:
 * - Strings are passed as (pointer, length) pairs, UTF-8 encoded
 * - DOM elements are identified by numeric IDs
 * - Event callbacks use indices into a callback table
 * - Memory is managed by WASM, JS only reads from it
 */

/**
 * Text decoder for reading UTF-8 strings from WASM memory
 */
const textDecoder = new TextDecoder('utf-8');

/**
 * Text encoder for writing UTF-8 strings to WASM memory (if needed)
 */
const textEncoder = new TextEncoder();

/**
 * Element ID tracking
 * Maps numeric IDs to actual DOM elements
 */
let nextElementId = 1;
const elementMap = new Map();
const INVALID_ELEMENT = 0;

/**
 * Timer ID tracking
 * Maps WASM timer IDs to browser timer IDs
 */
let nextTimerId = 1;
const timerMap = new Map();

/**
 * Register a DOM element and get its ID
 *
 * @param {HTMLElement} element - DOM element to register
 * @returns {number} Element ID
 */
function registerElement(element) {
    const id = nextElementId++;
    elementMap.set(id, element);
    return id;
}

/**
 * Get a DOM element by ID
 *
 * @param {number} id - Element ID
 * @returns {HTMLElement|null} DOM element or null if not found
 */
function getElement(id) {
    return elementMap.get(id) || null;
}

/**
 * Read a UTF-8 string from WASM memory
 *
 * @param {WebAssembly.Memory} memory - WASM memory instance
 * @param {number} ptr - Pointer to string data
 * @param {number} len - Length of string in bytes
 * @returns {string} Decoded string
 */
function readString(memory, ptr, len) {
    const bytes = new Uint8Array(memory.buffer, ptr, len);
    return textDecoder.decode(bytes);
}

/**
 * Create import object with all runtime functions
 *
 * @param {WebAssembly.Instance} instance - WASM instance (for callbacks)
 * @returns {object} Import object for WebAssembly.instantiate
 */
function createImports(instance = null) {
    return {
        env: {
            // ===== Console Functions =====

            /**
             * Log a message to the console
             *
             * @param {number} ptr - Pointer to UTF-8 string
             * @param {number} len - String length in bytes
             */
            console_log: (ptr, len) => {
                if (!instance) return;
                const msg = readString(instance.exports.memory, ptr, len);
                console.log('[WASM]', msg);
            },

            /**
             * Log an error message to the console
             *
             * @param {number} ptr - Pointer to UTF-8 string
             * @param {number} len - String length in bytes
             */
            console_error: (ptr, len) => {
                if (!instance) return;
                const msg = readString(instance.exports.memory, ptr, len);
                console.error('[WASM ERROR]', msg);
            },

            /**
             * Log a warning message to the console
             *
             * @param {number} ptr - Pointer to UTF-8 string
             * @param {number} len - String length in bytes
             */
            console_warn: (ptr, len) => {
                if (!instance) return;
                const msg = readString(instance.exports.memory, ptr, len);
                console.warn('[WASM WARN]', msg);
            },

            // ===== Timer Functions =====

            /**
             * Set a timeout to call a WASM callback after a delay
             *
             * The callback is invoked by calling the exported function
             * `__timer_callback_{callback_index}` from WASM.
             *
             * @param {number} callback_index - Index of callback in WASM
             * @param {number} delay_ms - Delay in milliseconds
             * @returns {number} Timer ID (can be used with clear_timeout)
             */
            js_set_timeout: (callback_index, delay_ms) => {
                const timerId = nextTimerId++;
                const browserId = setTimeout(() => {
                    timerMap.delete(timerId);
                    if (instance && instance.exports[`__timer_callback_${callback_index}`]) {
                        instance.exports[`__timer_callback_${callback_index}`]();
                    }
                }, delay_ms);
                timerMap.set(timerId, browserId);
                return timerId;
            },

            /**
             * Clear a previously set timeout
             *
             * @param {number} timer_id - Timer ID returned by js_set_timeout
             */
            js_clear_timeout: (timer_id) => {
                const browserId = timerMap.get(timer_id);
                if (browserId !== undefined) {
                    clearTimeout(browserId);
                    timerMap.delete(timer_id);
                }
            },

            /**
             * Set an interval to call a WASM callback repeatedly
             *
             * @param {number} callback_index - Index of callback in WASM
             * @param {number} interval_ms - Interval in milliseconds
             * @returns {number} Timer ID (can be used with clear_interval)
             */
            js_set_interval: (callback_index, interval_ms) => {
                const timerId = nextTimerId++;
                const browserId = setInterval(() => {
                    if (instance && instance.exports[`__timer_callback_${callback_index}`]) {
                        instance.exports[`__timer_callback_${callback_index}`]();
                    }
                }, interval_ms);
                timerMap.set(timerId, browserId);
                return timerId;
            },

            /**
             * Clear a previously set interval
             *
             * @param {number} timer_id - Timer ID returned by js_set_interval
             */
            js_clear_interval: (timer_id) => {
                const browserId = timerMap.get(timer_id);
                if (browserId !== undefined) {
                    clearInterval(browserId);
                    timerMap.delete(timer_id);
                }
            },

            // ===== DOM Functions =====

            /**
             * Create a DOM element
             *
             * @param {number} tag_ptr - Pointer to tag name string
             * @param {number} tag_len - Tag name length
             * @returns {number} Element ID or INVALID_ELEMENT on failure
             */
            dom_create_element: (tag_ptr, tag_len) => {
                if (!instance) return INVALID_ELEMENT;
                try {
                    const tag = readString(instance.exports.memory, tag_ptr, tag_len);
                    const element = document.createElement(tag);
                    return registerElement(element);
                } catch (e) {
                    console.error('Failed to create element:', e);
                    return INVALID_ELEMENT;
                }
            },

            /**
             * Set an attribute on a DOM element
             *
             * Uses setAttribute() to set the attribute, not innerHTML.
             *
             * @param {number} elem_id - Element ID
             * @param {number} key_ptr - Pointer to attribute name
             * @param {number} key_len - Attribute name length
             * @param {number} val_ptr - Pointer to attribute value
             * @param {number} val_len - Attribute value length
             */
            dom_set_attribute: (elem_id, key_ptr, key_len, val_ptr, val_len) => {
                if (!instance) return;
                const element = getElement(elem_id);
                if (!element) return;

                const key = readString(instance.exports.memory, key_ptr, key_len);
                const value = readString(instance.exports.memory, val_ptr, val_len);
                element.setAttribute(key, value);
            },

            /**
             * Set the text content of a DOM element
             *
             * Uses textContent, not innerHTML, for security.
             *
             * @param {number} elem_id - Element ID
             * @param {number} text_ptr - Pointer to text string
             * @param {number} text_len - Text length
             */
            dom_set_text_content: (elem_id, text_ptr, text_len) => {
                if (!instance) return;
                const element = getElement(elem_id);
                if (!element) return;

                const text = readString(instance.exports.memory, text_ptr, text_len);
                element.textContent = text;
            },

            /**
             * Append a child element to a parent
             *
             * Uses appendChild() DOM method.
             *
             * @param {number} parent_id - Parent element ID
             * @param {number} child_id - Child element ID
             */
            dom_append_child: (parent_id, child_id) => {
                const parent = getElement(parent_id);
                const child = getElement(child_id);
                if (parent && child) {
                    parent.appendChild(child);
                }
            },

            /**
             * Add an event listener to a DOM element
             *
             * When the event fires, calls the WASM callback function
             * `__event_callback_{callback_index}`.
             *
             * @param {number} elem_id - Element ID
             * @param {number} event_ptr - Pointer to event name
             * @param {number} event_len - Event name length
             * @param {number} callback_index - Callback index in WASM
             */
            dom_add_event_listener: (elem_id, event_ptr, event_len, callback_index) => {
                if (!instance) return;
                const element = getElement(elem_id);
                if (!element) return;

                const eventName = readString(instance.exports.memory, event_ptr, event_len);
                element.addEventListener(eventName, (event) => {
                    // Call WASM callback if it exists
                    const callbackName = `__event_callback_${callback_index}`;
                    if (instance.exports[callbackName]) {
                        instance.exports[callbackName](elem_id);
                    }
                });
            },

            /**
             * Query for an existing DOM element by selector
             *
             * Uses document.querySelector() to find the element.
             *
             * @param {number} selector_ptr - Pointer to CSS selector
             * @param {number} selector_len - Selector length
             * @returns {number} Element ID or INVALID_ELEMENT if not found
             */
            dom_query_selector: (selector_ptr, selector_len) => {
                if (!instance) return INVALID_ELEMENT;
                try {
                    const selector = readString(instance.exports.memory, selector_ptr, selector_len);
                    const element = document.querySelector(selector);
                    if (element) {
                        return registerElement(element);
                    }
                    return INVALID_ELEMENT;
                } catch (e) {
                    console.error('Query selector failed:', e);
                    return INVALID_ELEMENT;
                }
            },

            /**
             * Remove a DOM element from the document
             *
             * Uses element.remove() to remove from DOM.
             *
             * @param {number} elem_id - Element ID
             */
            dom_remove_element: (elem_id) => {
                const element = getElement(elem_id);
                if (element) {
                    element.remove();
                    elementMap.delete(elem_id);
                }
            },

            /**
             * Remove a child element from its parent
             *
             * Uses parent.removeChild() DOM method.
             *
             * @param {number} parent_id - Parent element ID
             * @param {number} child_id - Child element ID
             */
            dom_remove_child: (parent_id, child_id) => {
                const parent = getElement(parent_id);
                const child = getElement(child_id);
                if (parent && child) {
                    parent.removeChild(child);
                }
            },

            /**
             * Insert a child before another child
             *
             * Uses parent.insertBefore() DOM method.
             *
             * @param {number} parent_id - Parent element ID
             * @param {number} new_child_id - New child to insert
             * @param {number} reference_id - Reference child to insert before
             */
            dom_insert_before: (parent_id, new_child_id, reference_id) => {
                const parent = getElement(parent_id);
                const newChild = getElement(new_child_id);
                const reference = getElement(reference_id);
                if (parent && newChild && reference) {
                    parent.insertBefore(newChild, reference);
                }
            },

            /**
             * Set a property on a DOM element
             *
             * Sets element[property] = value directly.
             * Useful for properties like 'value', 'checked', etc.
             *
             * @param {number} elem_id - Element ID
             * @param {number} prop_ptr - Pointer to property name
             * @param {number} prop_len - Property name length
             * @param {number} val_ptr - Pointer to value
             * @param {number} val_len - Value length
             */
            dom_set_property: (elem_id, prop_ptr, prop_len, val_ptr, val_len) => {
                if (!instance) return;
                const element = getElement(elem_id);
                if (!element) return;

                const property = readString(instance.exports.memory, prop_ptr, prop_len);
                const value = readString(instance.exports.memory, val_ptr, val_len);
                element[property] = value;
            },

            /**
             * Set a CSS style property on an element
             *
             * Sets element.style[property] = value.
             *
             * @param {number} elem_id - Element ID
             * @param {number} prop_ptr - Pointer to style property name
             * @param {number} prop_len - Property name length
             * @param {number} val_ptr - Pointer to style value
             * @param {number} val_len - Value length
             */
            dom_set_style: (elem_id, prop_ptr, prop_len, val_ptr, val_len) => {
                if (!instance) return;
                const element = getElement(elem_id);
                if (!element) return;

                const property = readString(instance.exports.memory, prop_ptr, prop_len);
                const value = readString(instance.exports.memory, val_ptr, val_len);
                element.style[property] = value;
            },

            // Placeholder for println (from WASM codegen tests)
            println: (val) => {
                console.log('[WASM println]', val);
            }
        }
    };
}

/**
 * Load and instantiate a WebAssembly module
 *
 * This function:
 * 1. Creates the import object with all runtime functions
 * 2. Instantiates the WASM module with imports
 * 3. Returns the instance for use
 *
 * @param {ArrayBuffer|Uint8Array} wasmBytes - WASM module bytes
 * @returns {Promise<WebAssembly.Instance>} WASM instance
 */
export async function loadWasmModule(wasmBytes) {
    // Create a temporary instance reference for the imports
    let instanceRef = null;
    const imports = createImports();

    // Instantiate the module
    const result = await WebAssembly.instantiate(wasmBytes, imports);
    instanceRef = result.instance;

    // Update the imports to have access to the instance
    // (for reading memory and calling callbacks)
    const finalImports = createImports(instanceRef);

    // Re-instantiate with the proper imports that have instance access
    const finalResult = await WebAssembly.instantiate(wasmBytes, finalImports);

    return finalResult.instance;
}

/**
 * Mount a Blang component to a DOM element
 *
 * This function:
 * 1. Finds the container element by selector
 * 2. Registers it in the element map
 * 3. Calls the component's init function with the container ID
 *
 * Component initialization function naming convention:
 * - For component "counter": expects `init_counter(container_id)` export
 * - For component "todo_list": expects `init_todo_list(container_id)` export
 *
 * @param {string} selector - CSS selector for container element
 * @param {string} componentName - Name of the component to mount
 * @param {WebAssembly.Instance} wasmInstance - WASM instance with component
 * @returns {number} Container element ID
 * @throws {Error} If container element is not found
 */
export function mountComponent(selector, componentName, wasmInstance) {
    // Find the container element
    const container = document.querySelector(selector);
    if (!container) {
        throw new Error(`Container element not found: ${selector}`);
    }

    // Register container and get its ID
    const containerId = registerElement(container);

    // Call the component's init function
    const initFunctionName = `init_${componentName}`;
    if (wasmInstance.exports[initFunctionName]) {
        wasmInstance.exports[initFunctionName](containerId);
        console.log(`Mounted component '${componentName}' to ${selector}`);
    } else {
        console.warn(
            `Component '${componentName}' does not export ${initFunctionName}. ` +
            `Available exports:`, Object.keys(wasmInstance.exports)
        );
    }

    return containerId;
}

/**
 * Reactive state management for components
 *
 * This provides a simple reactive system where:
 * 1. Components can register state change handlers
 * 2. When state changes, handlers are called
 * 3. Handlers can trigger re-renders
 */
export class ComponentState {
    constructor(wasmInstance) {
        this.instance = wasmInstance;
        this.state = new Map();
        this.handlers = new Map();
    }

    /**
     * Set a state value and trigger updates
     *
     * @param {string} key - State key
     * @param {*} value - New value
     */
    setState(key, value) {
        const oldValue = this.state.get(key);
        if (oldValue !== value) {
            this.state.set(key, value);
            this.notifyHandlers(key, value, oldValue);
        }
    }

    /**
     * Get a state value
     *
     * @param {string} key - State key
     * @returns {*} State value
     */
    getState(key) {
        return this.state.get(key);
    }

    /**
     * Register a handler for state changes
     *
     * @param {string} key - State key to watch
     * @param {Function} handler - Handler function(newValue, oldValue)
     */
    onStateChange(key, handler) {
        if (!this.handlers.has(key)) {
            this.handlers.set(key, []);
        }
        this.handlers.get(key).push(handler);
    }

    /**
     * Notify all handlers for a state key
     *
     * @private
     * @param {string} key - State key
     * @param {*} newValue - New value
     * @param {*} oldValue - Old value
     */
    notifyHandlers(key, newValue, oldValue) {
        const handlers = this.handlers.get(key);
        if (handlers) {
            handlers.forEach(handler => handler(newValue, oldValue));
        }
    }

    /**
     * Trigger a re-render by calling the WASM render function
     *
     * @param {string} componentName - Component name
     * @param {number} containerId - Container element ID
     */
    render(componentName, containerId) {
        const renderFn = `render_${componentName}`;
        if (this.instance.exports[renderFn]) {
            this.instance.exports[renderFn](containerId);
        }
    }
}

/**
 * Create a new component state manager
 *
 * @param {WebAssembly.Instance} wasmInstance - WASM instance
 * @returns {ComponentState} State manager
 */
export function createComponentState(wasmInstance) {
    return new ComponentState(wasmInstance);
}

// Export element management for advanced use cases
export { registerElement, getElement, INVALID_ELEMENT };
