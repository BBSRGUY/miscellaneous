# Todo App Example

A full-featured todo list application demonstrating advanced component mode features.

## What This Example Demonstrates

- Component composition (TodoApp, TodoItem components)
- Complex state management
- List rendering with `for` loops
- Form handling and input events
- Conditional rendering
- Component props and callbacks
- Filtering and computed values
- Local storage persistence (conceptual)

## The Code

This example implements a complete todo application with:

1. **TodoApp Component**: Main application container
   - Manages the list of todos
   - Handles adding new todos
   - Provides filtering (All, Active, Completed)
   - Shows statistics

2. **TodoItem Component**: Individual todo item
   - Displays todo text
   - Toggle completion status
   - Delete todo
   - Edit todo (inline editing)

## File Structure

```
todo_app/
├── README.md          # This file
└── main.blang         # Todo app implementation
```

## Features

- ✅ Add new todos
- ✅ Mark todos as complete/incomplete
- ✅ Delete todos
- ✅ Filter by status (All, Active, Completed)
- ✅ Show active todo count
- ✅ Clear all completed todos
- ✅ Edit todos inline
- ✅ Responsive design

## Running the Example

### Using the CLI

```bash
# Navigate to this directory
cd examples/todo_app

# Start the development server
blang dev

# Open http://localhost:3000 in your browser
```

### Building for Production

```bash
# Compile to WASM
blang compile main.blang -o todo.wasm

# Create a bundle
blang bundle main.blang --out-dir dist
```

## Understanding the Code

### Data Structure

```blang
struct Todo {
    id: i32,
    text: str,
    completed: bool,
}
```

Each todo is a struct with a unique ID, text content, and completion status.

### Component Composition

```blang
component TodoApp {
    view {
        {for todo in filtered_todos() {
            <TodoItem
                todo={todo}
                on_toggle={|| toggle_todo(todo.id)}
                on_delete={|| delete_todo(todo.id)}
            />
        }}
    }
}
```

The main app renders child `TodoItem` components, passing props and callbacks.

### List Rendering

```blang
{for todo in todos {
    <TodoItem todo={todo} />
}}
```

The `for` loop renders a component for each todo in the list.

### Conditional Rendering

```blang
{if todos.is_empty() {
    <p>"No todos yet!"</p>
} else {
    <TodoList todos={todos} />
}}
```

Show different UI based on application state.

### Computed Values

```blang
fn filtered_todos() -> Vec<Todo> {
    match filter {
        "active" => todos.iter().filter(|t| !t.completed).collect(),
        "completed" => todos.iter().filter(|t| t.completed).collect(),
        _ => todos.clone(),
    }
}

fn active_count() -> i32 {
    todos.iter().filter(|t| !t.completed).count()
}
```

Derive values from state instead of storing them.

## Code Highlights

### Adding Todos

```blang
fn add_todo() {
    if !new_todo_text.is_empty() {
        todos.push(Todo {
            id: next_id,
            text: new_todo_text.clone(),
            completed: false,
        });
        next_id = next_id + 1;
        new_todo_text = String::new();
    }
}
```

### Toggling Completion

```blang
fn toggle_todo(id: i32) {
    for todo in &mut todos {
        if todo.id == id {
            todo.completed = !todo.completed;
            break;
        }
    }
}
```

### Filtering

```blang
fn set_filter(new_filter: str) {
    filter = new_filter;
}
```

## Next Steps

After understanding this example, explore:

- **[docs/component_mode.md](../../docs/component_mode.md)**: Advanced component patterns
- **[docs/language_overview.md](../../docs/language_overview.md)**: Full language reference
- **[job_sync](../job_sync/README.md)**: Script mode example with data pipelines

## Extending This Example

Ideas for extending this todo app:

1. **Categories/Tags**: Add tags or categories to todos
2. **Due Dates**: Add date pickers and sort by due date
3. **Priority Levels**: High, medium, low priority
4. **Search**: Filter todos by text search
5. **Animations**: Add transitions when adding/removing todos
6. **Persistence**: Save to localStorage or backend API
7. **Drag and Drop**: Reorder todos by dragging
8. **Subtasks**: Nested todo items

## Learning Resources

- [Component Mode Guide](../../docs/component_mode.md) - Complete component documentation
- [Language Overview](../../docs/language_overview.md) - Core Blang syntax
- [Counter Example](../counter_app/README.md) - Simpler starting point
