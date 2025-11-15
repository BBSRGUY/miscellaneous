# Counter App Example

A simple reactive counter component demonstrating Blang's component mode basics.

## What This Example Demonstrates

- Basic component structure
- State management
- Event handling
- View templates with interpolation
- Component styling

## The Code

This example consists of a single component `Counter` that:

1. Maintains a count in component state
2. Provides increment, decrement, and reset buttons
3. Displays the current count
4. Includes scoped styles for the component

## File Structure

```
counter_app/
├── README.md          # This file
└── main.blang         # Counter component implementation
```

## Running the Example

### Using the CLI

```bash
# Navigate to this directory
cd examples/counter_app

# Start the development server
blang dev

# Open http://localhost:3000 in your browser
```

### Building for Production

```bash
# Compile to WASM
blang compile main.blang -o counter.wasm

# Create a bundle (WASM + HTML + JS runtime)
blang bundle main.blang --out-dir dist

# Serve the dist folder
```

## Understanding the Code

### Component State

```blang
state count: i32 = 0;
```

The counter uses a single state variable `count` initialized to 0. When this state changes, the UI automatically updates.

### Event Handlers

```blang
fn increment() {
    count = count + 1;
}
```

Event handler methods modify state. These are called from the view template using `on:click` event bindings.

### View Template

```blang
view {
    <div class="counter">
        <h1>"Counter Example"</h1>
        <div class="display">{count}</div>
        <button on:click={increment}>"+1"</button>
    </div>
}
```

The view template uses JSX-like syntax with:
- String literals in quotes
- State interpolation with `{count}`
- Event bindings with `on:click={handler}`

### Scoped Styles

```blang
style {
    .counter {
        max-width: 400px;
        margin: 2rem auto;
    }
}
```

Styles are scoped to the component and won't leak to other components.

## Next Steps

After understanding this example, check out:

- **[todo_app](../todo_app/README.md)**: A more complex example with lists, forms, and multiple components
- **[docs/component_mode.md](../../docs/component_mode.md)**: Complete component mode documentation

## Learning Resources

- [Language Overview](../../docs/language_overview.md) - Core Blang syntax and features
- [Component Mode Guide](../../docs/component_mode.md) - Comprehensive component documentation
- [Blang README](../../README.md) - Project overview and setup
