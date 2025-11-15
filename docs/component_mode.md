# Component Mode - Building Reactive UI Components

A comprehensive guide to building user interfaces with Blang's component mode.

## Table of Contents

- [Introduction](#introduction)
- [Component Basics](#component-basics)
- [State Management](#state-management)
- [View Templates](#view-templates)
- [Event Handling](#event-handling)
- [Lifecycle Hooks](#lifecycle-hooks)
- [Styling](#styling)
- [Props and Composition](#props-and-composition)
- [Computed Values](#computed-values)
- [Effects and Side Effects](#effects-and-side-effects)
- [Best Practices](#best-practices)

## Introduction

Component mode is Blang's approach to building reactive user interfaces. Components are self-contained units that encapsulate state, view logic, and styling.

### Key Concepts

- **Reactivity**: State changes automatically update the UI
- **Composability**: Components can be nested and reused
- **Type Safety**: Full type checking for state and props
- **Performance**: Compiled to efficient WebAssembly
- **Locality**: View, logic, and styles in one place

## Component Basics

### Minimal Component

```blang
component Hello {
    view {
        <div>"Hello, World!"</div>
    }
}
```

### Component Structure

A component can have these sections (all optional except `view`):

```blang
component MyComponent {
    // Props (inputs from parent)
    props {
        title: str,
        count: i32,
    }

    // Component state
    state counter: i32 = 0;
    state message: str = "";

    // Lifecycle hooks
    on_mount() {
        println("Component mounted");
    }

    on_update() {
        println("Component updated");
    }

    on_unmount() {
        println("Component will unmount");
    }

    // Methods
    fn increment() {
        counter = counter + 1;
    }

    fn reset() {
        counter = 0;
    }

    // Required: View template
    view {
        <div>
            <h1>{title}</h1>
            <p>"Counter: " {counter}</p>
            <button on:click={increment}>"Increment"</button>
        </div>
    }

    // Optional: Component styles
    style {
        div {
            padding: 20px;
            background: #f0f0f0;
        }
    }
}
```

## State Management

### Declaring State

```blang
component Counter {
    // State with initial value
    state count: i32 = 0;
    state message: str = "Hello";
    state is_active: bool = false;

    // Multiple state variables
    state {
        todos: Vec<str> = vec![],
        filter: str = "all",
        input: str = "",
    }

    view {
        <div>{count}</div>
    }
}
```

### Updating State

```blang
component TodoList {
    state todos: Vec<str> = vec![];
    state input: str = "";

    fn add_todo() {
        // Direct assignment
        todos.push(input);
        input = "";
    }

    fn clear_all() {
        todos = vec![];
    }

    fn update_input(new_value: str) {
        input = new_value;
    }

    view {
        <div>
            <input value={input} on:input={|e| update_input(e.target.value)} />
            <button on:click={add_todo}>"Add"</button>
        </div>
    }
}
```

### State Best Practices

1. **Keep state minimal**: Only store what's necessary
2. **Derive when possible**: Compute values instead of storing them
3. **Normalize data**: Avoid nested redundancy
4. **Use appropriate types**: Match state type to data

```blang
component UserList {
    // Good: Store data, compute derived values
    state users: Vec<User> = vec![];

    fn active_users() -> Vec<User> {
        users.iter().filter(|u| u.is_active).collect()
    }

    fn user_count() -> i32 {
        users.len()
    }

    view {
        <div>
            <p>"Total users: " {user_count()}</p>
            <p>"Active users: " {active_users().len()}</p>
        </div>
    }
}
```

## View Templates

### Basic Syntax

```blang
component Examples {
    state name: str = "Alice";
    state count: i32 = 42;

    view {
        // Text content
        <div>"Hello, World!"</div>

        // Interpolation
        <p>"Hello, " {name}</p>

        // Multiple children
        <div>
            <h1>"Title"</h1>
            <p>"Paragraph"</p>
        </div>

        // Attributes
        <div class="container" id="main">
            "Content"
        </div>

        // Dynamic attributes
        <div class={if count > 10 { "large" } else { "small" }}>
            {count}
        </div>
    }
}
```

### Conditional Rendering

```blang
component Conditional {
    state is_logged_in: bool = false;
    state user_role: str = "guest";

    view {
        <div>
            // If condition
            {if is_logged_in {
                <p>"Welcome back!"</p>
            }}

            // If-else
            {if is_logged_in {
                <button>"Logout"</button>
            } else {
                <button>"Login"</button>
            }}

            // Multiple conditions
            {if user_role == "admin" {
                <div>"Admin Panel"</div>
            } else if user_role == "user" {
                <div>"User Dashboard"</div>
            } else {
                <div>"Guest View"</div>
            }}
        </div>
    }
}
```

### List Rendering

```blang
component TodoList {
    state todos: Vec<Todo> = vec![
        Todo { id: 1, text: "Learn Blang", done: false },
        Todo { id: 2, text: "Build app", done: false },
    ];

    view {
        <ul>
            // For loop
            {for todo in todos {
                <li>
                    <input type="checkbox" checked={todo.done} />
                    <span>{todo.text}</span>
                </li>
            }}
        </ul>
    }
}
```

### Fragments

```blang
component MultiRoot {
    view {
        // Fragment for multiple root elements
        <>
            <header>"Header"</header>
            <main>"Content"</main>
            <footer>"Footer"</footer>
        </>
    }
}
```

## Event Handling

### Click Events

```blang
component Buttons {
    state count: i32 = 0;

    fn increment() {
        count += 1;
    }

    fn decrement() {
        count -= 1;
    }

    fn reset() {
        count = 0;
    }

    view {
        <div>
            <button on:click={increment}>"+"</button>
            <button on:click={decrement}>"-"</button>
            <button on:click={reset}>"Reset"</button>
            <p>"Count: " {count}</p>
        </div>
    }
}
```

### Input Events

```blang
component Form {
    state name: str = "";
    state email: str = "";
    state age: i32 = 0;

    fn handle_name_input(e: InputEvent) {
        name = e.target.value;
    }

    fn handle_submit(e: Event) {
        e.prevent_default();
        println("Submitted: " + name + ", " + email);
    }

    view {
        <form on:submit={handle_submit}>
            <input
                type="text"
                value={name}
                on:input={handle_name_input}
                placeholder="Name"
            />

            <input
                type="email"
                value={email}
                on:input={|e| email = e.target.value}
                placeholder="Email"
            />

            <button type="submit">"Submit"</button>
        </form>
    }
}
```

### Event Modifiers

```blang
component EventModifiers {
    view {
        // Prevent default
        <form on:submit|prevent={handle_submit}>
            <button>"Submit"</button>
        </form>

        // Stop propagation
        <div on:click={outer}>
            <button on:click|stop={inner}>"Click"</button>
        </div>

        // Once (handler runs only once)
        <button on:click|once={init}>"Initialize"</button>

        // Capture phase
        <div on:click|capture={handler}>
            "Content"
        </div>
    }
}
```

## Lifecycle Hooks

### On Mount

Runs when component is added to the DOM:

```blang
component DataFetcher {
    state data: Option<Data> = None;
    state loading: bool = false;

    on_mount() {
        loading = true;
        fetch_data().then(|result| {
            data = Some(result);
            loading = false;
        });
    }

    view {
        {if loading {
            <div>"Loading..."</div>
        } else if let Some(d) = data {
            <div>{d.display()}</div>
        } else {
            <div>"No data"</div>
        }}
    }
}
```

### On Update

Runs when component state changes:

```blang
component Logger {
    state count: i32 = 0;

    on_update() {
        println("Count changed to: " + count.to_string());
    }

    fn increment() {
        count += 1;
    }

    view {
        <button on:click={increment}>"Count: " {count}</button>
    }
}
```

### On Unmount

Runs when component is removed from DOM:

```blang
component Timer {
    state interval_id: Option<i32> = None;

    on_mount() {
        interval_id = Some(set_interval(|| {
            println("tick");
        }, 1000));
    }

    on_unmount() {
        if let Some(id) = interval_id {
            clear_interval(id);
        }
    }

    view {
        <div>"Timer running"</div>
    }
}
```

## Styling

### Scoped Styles

```blang
component StyledButton {
    props {
        variant: str = "primary",
    }

    view {
        <button class={variant}>
            "Click me"
        </button>
    }

    style {
        button {
            padding: 10px 20px;
            border: none;
            border-radius: 4px;
            font-size: 16px;
            cursor: pointer;
        }

        .primary {
            background: #007bff;
            color: white;
        }

        .secondary {
            background: #6c757d;
            color: white;
        }

        .danger {
            background: #dc3545;
            color: white;
        }

        button:hover {
            opacity: 0.8;
        }
    }
}
```

### Dynamic Styles

```blang
component DynamicStyles {
    props {
        color: str = "#000",
        size: i32 = 16,
    }

    view {
        <div class="dynamic" style={format("color: {}; font-size: {}px", color, size)}>
            "Styled text"
        </div>
    }

    style {
        .dynamic {
            transition: all 0.3s ease;
        }
    }
}
```

## Props and Composition

### Defining Props

```blang
component UserCard {
    props {
        user: User,
        show_email: bool = false,  // Optional with default
        on_click: fn() = || {},    // Optional callback
    }

    view {
        <div class="user-card" on:click={on_click}>
            <h3>{user.name}</h3>
            {if show_email {
                <p>{user.email}</p>
            }}
        </div>
    }
}
```

### Using Child Components

```blang
component UserList {
    state users: Vec<User> = load_users();

    fn handle_user_click(user: User) {
        println("Clicked: " + user.name);
    }

    view {
        <div>
            {for user in users {
                <UserCard
                    user={user}
                    show_email={true}
                    on_click={|| handle_user_click(user)}
                />
            }}
        </div>
    }
}
```

### Slots (Children)

```blang
component Card {
    props {
        title: str,
    }

    view {
        <div class="card">
            <header>
                <h2>{title}</h2>
            </header>
            <div class="content">
                <slot />  // Children go here
            </div>
        </div>
    }
}

// Usage
component App {
    view {
        <Card title="My Card">
            <p>"This is the card content"</p>
            <button>"Action"</button>
        </Card>
    }
}
```

## Computed Values

### Computed Properties

```blang
component TodoList {
    state todos: Vec<Todo> = vec![];
    state filter: str = "all";

    fn filtered_todos() -> Vec<Todo> {
        match filter {
            "active" => todos.iter().filter(|t| !t.done).collect(),
            "completed" => todos.iter().filter(|t| t.done).collect(),
            _ => todos.clone(),
        }
    }

    fn active_count() -> i32 {
        todos.iter().filter(|t| !t.done).count()
    }

    view {
        <div>
            <p>"Active: " {active_count()}</p>
            <ul>
                {for todo in filtered_todos() {
                    <li>{todo.text}</li>
                }}
            </ul>
        </div>
    }
}
```

### Memoization

```blang
component Expensive {
    state input: str = "";

    memo expensive_computation(text: str) -> str {
        // Only recomputed when text changes
        process_heavy_operation(text)
    }

    view {
        <div>
            <input value={input} on:input={|e| input = e.target.value} />
            <p>{expensive_computation(input)}</p>
        </div>
    }
}
```

## Effects and Side Effects

### Effect Hook

```blang
component DataSync {
    state local_data: str = "";
    state sync_status: str = "idle";

    effect on local_data {
        // Runs whenever local_data changes
        sync_status = "syncing";
        save_to_server(local_data).then(|| {
            sync_status = "saved";
        });
    }

    view {
        <div>
            <textarea value={local_data} on:input={|e| local_data = e.target.value} />
            <p>"Status: " {sync_status}</p>
        </div>
    }
}
```

## Best Practices

### Component Organization

```blang
// Good: Small, focused components
component TodoItem {
    props {
        todo: Todo,
        on_toggle: fn(i32),
        on_delete: fn(i32),
    }

    view {
        <li>
            <input
                type="checkbox"
                checked={todo.done}
                on:change={|| on_toggle(todo.id)}
            />
            <span>{todo.text}</span>
            <button on:click={|| on_delete(todo.id)}>"Delete"</button>
        </li>
    }
}

component TodoList {
    state todos: Vec<Todo> = vec![];

    fn toggle_todo(id: i32) {
        // Toggle logic
    }

    fn delete_todo(id: i32) {
        todos = todos.iter().filter(|t| t.id != id).collect();
    }

    view {
        <ul>
            {for todo in todos {
                <TodoItem
                    todo={todo}
                    on_toggle={toggle_todo}
                    on_delete={delete_todo}
                />
            }}
        </ul>
    }
}
```

### State Management Patterns

1. **Lift state up**: Share state among siblings by moving it to parent
2. **Keep state local**: Don't lift state unnecessarily
3. **Single source of truth**: Each piece of state has one owner
4. **Derived state**: Compute rather than store when possible

### Performance Tips

- Use `memo` for expensive computations
- Avoid unnecessary re-renders with proper prop types
- Keep component tree shallow when possible
- Use keys for list items (coming soon)

---

For more information, see:

- [Language Overview](language_overview.md) - Core language features
- [Script Mode](script_mode.md) - Data processing and jobs
- [Unsafe Blocks](unsafe_blocks.md) - Low-level programming
