# Forge Store

Database and blob storage layer for the Forge platform.

## Features

- **SQLite database** with automatic migrations
- **Repository pattern** for clean data access
- **Blob storage** for large files (models, datasets, checkpoints)
- **Full CRUD operations** for all entities
- **Comprehensive test coverage**

## Quick Start

```rust
use forge_store::{Store, entities::Model, BlobCategory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize store
    let store = Store::new_with_file("forge.db", "data").await?;

    // Create a model
    let mut model = Model::new(
        "llama-2-7b".to_string(),
        "llama".to_string(),
        "gguf".to_string(),
    );
    store.models().create(&model).await?;

    // Store model file
    let data = std::fs::read("model.gguf")?;
    let path = store.blob_storage().store_blob_named(
        BlobCategory::Models,
        "llama-2-7b.gguf",
        &data,
    )?;

    model.path = Some(path);
    store.models().update(&model).await?;

    // List all models
    let models = store.models().list().await?;
    for model in models {
        println!("Model: {}", model.name);
    }

    store.close().await;
    Ok(())
}
```

## Entities

The store provides repositories for:

- **Models** - LLM model metadata
- **Sessions** - Chat sessions
- **Messages** - Chat messages (linked to sessions)
- **Jobs** - Background tasks (training, inference)
- **Documents** - RAG documents

See [entities.rs](src/entities.rs) for full details.

## Database Schema

Migrations are automatically applied on startup. See `migrations/` directory for SQL schema.

## Testing

```bash
cargo test -p forge_store
```

All tests use in-memory databases and temporary directories.

## Documentation

See the [Storage documentation](../../docs/storage.md) for detailed usage examples.
