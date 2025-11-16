# Forge Storage Layer

The storage layer in Forge provides persistent storage for all application data through two primary mechanisms:

1. **SQLite Database** - Structured data (models, sessions, messages, jobs, documents)
2. **Blob Storage** - Large files (model weights, datasets, checkpoints, logs)

## Architecture

### Database Schema

The database consists of five main tables:

**models** - LLM model metadata
- `id`: Unique identifier (UUID)
- `name`: Model name (unique)
- `path`: Relative path to model blob
- `backend`: Backend type (e.g., "llama", "echo")
- `format`: Model format (e.g., "gguf", "safetensors")
- `size_bytes`: File size in bytes
- `quantization`: Quantization method (e.g., "Q4_K_M")
- `tags`: JSON array of tags
- `created_at`, `updated_at`: Timestamps

**sessions** - Chat sessions
- `id`: Unique identifier
- `name`: Session name
- `model_id`: Foreign key to models (nullable)
- `config_json`: Session configuration
- `created_at`, `updated_at`: Timestamps

**messages** - Chat messages
- `id`: Unique identifier
- `session_id`: Foreign key to sessions (CASCADE on delete)
- `role`: Message role ("user", "assistant", "system")
- `content`: Message text
- `meta_json`: Additional metadata
- `created_at`: Timestamp

**jobs** - Background tasks
- `id`: Unique identifier
- `kind`: Job type ("inference", "training", "embedding")
- `status`: Current status ("queued", "running", "completed", "failed", "cancelled")
- `progress`: Progress (0.0 to 1.0)
- `config_json`: Job configuration
- `logs_path`: Path to log file
- `error_message`: Error details (if failed)
- `created_at`, `updated_at`, `started_at`, `completed_at`: Timestamps

**documents** - RAG documents
- `id`: Unique identifier
- `source`: Source identifier (path, URL, etc.)
- `path`: Path to processed document blob
- `content`: Extracted text content
- `meta_json`: Document metadata
- `indexed_at`: Last indexing timestamp
- `created_at`, `updated_at`: Timestamps

### Blob Storage

Blob storage uses the filesystem with a configurable root directory. Files are organized into categories:

- `models/` - Model weight files
- `checkpoints/` - Training checkpoints
- `datasets/` - Training/fine-tuning datasets
- `logs/` - Job log files

## Usage

### Initialization

```rust
use forge_runtime::config::AppConfig;
use forge_store::Store;

// Load configuration
let config = AppConfig::load()?;

// Initialize store from configuration
let store = Store::from_config(
    config.db_path(),
    config.storage.data_dir.clone(),
).await?;
```

### CRUD Operations

#### Models

```rust
use forge_store::entities::Model;

// Create a model
let mut model = Model::new(
    "llama-2-7b".to_string(),
    "llama".to_string(),
    "gguf".to_string(),
);
model.quantization = Some("Q4_K_M".to_string());
model.set_tags(vec!["llama2".to_string(), "7b".to_string()]);

store.models().create(&model).await?;

// Read a model
let retrieved = store.models().get(&model.id).await?;
let by_name = store.models().get_by_name("llama-2-7b").await?;

// List all models
let models = store.models().list().await?;

// Update a model
model.backend = "llama-cpp".to_string();
store.models().update(&model).await?;

// Delete a model
store.models().delete(&model.id).await?;
```

#### Sessions and Messages

```rust
use forge_store::entities::{Session, Message};

// Create a session
let session = Session::new("My Chat".to_string(), Some(model.id.clone()));
store.sessions().create(&session).await?;

// Add messages
let user_msg = Message::new(
    session.id.clone(),
    "user".to_string(),
    "Hello!".to_string(),
);
store.messages().create(&user_msg).await?;

let asst_msg = Message::new(
    session.id.clone(),
    "assistant".to_string(),
    "Hi! How can I help?".to_string(),
);
store.messages().create(&asst_msg).await?;

// Retrieve messages for a session
let messages = store.messages().get_by_session(&session.id).await?;
```

#### Jobs

```rust
use forge_store::entities::Job;

// Create a job
let mut job = Job::new("training".to_string());
job.config_json = Some(r#"{"epochs": 10, "batch_size": 32}"#.to_string());
store.jobs().create(&job).await?;

// Update job status
job.start();
store.jobs().update(&job).await?;

// Update progress
job.progress = 0.5;
store.jobs().update(&job).await?;

// Complete job
job.complete();
store.jobs().update(&job).await?;

// Or fail job
job.fail("Out of memory".to_string());
store.jobs().update(&job).await?;

// List jobs by status
let running_jobs = store.jobs().list_by_status("running").await?;
```

#### Documents

```rust
use forge_store::entities::Document;

// Create a document
let mut doc = Document::new("file:///path/to/doc.pdf".to_string());
doc.content = Some("Extracted document text...".to_string());
doc.meta_json = Some(r#"{"title": "User Manual", "author": "Forge Team"}"#.to_string());
store.documents().create(&doc).await?;

// Mark as indexed
doc.mark_indexed();
store.documents().update(&doc).await?;
```

### Blob Storage

```rust
use forge_store::BlobCategory;

// Store a model file
let model_data = std::fs::read("path/to/model.gguf")?;
let blob_path = store.blob_storage().store_blob_named(
    BlobCategory::Models,
    "llama-2-7b.gguf",
    &model_data,
)?;

// Store blob path in model record
model.path = Some(blob_path.clone());
model.size_bytes = Some(model_data.len() as i64);
store.models().update(&model).await?;

// Retrieve blob
let data = store.blob_storage().get_blob(&blob_path)?;

// Get absolute path
let abs_path = store.blob_storage().get_blob_path(&blob_path);

// Check if blob exists
if store.blob_storage().blob_exists(&blob_path) {
    // Delete blob
    store.blob_storage().delete_blob(&blob_path)?;
}

// List blobs in a category
let model_blobs = store.blob_storage().list_blobs(BlobCategory::Models)?;
```

## Configuration Integration

The store integrates with the Forge configuration system:

```yaml
# configs/default.yaml
db:
  path: "forge.db"        # Relative to data_dir
  wal_mode: true          # Enable WAL for better concurrency
  pool_size: 5            # Connection pool size

storage:
  data_dir: "data"        # Base directory
  models_dir: "models"    # Relative to data_dir
  datasets_dir: "datasets"
  checkpoints_dir: "checkpoints"
```

Configuration is passed to the store:

```rust
use forge_runtime::config::AppConfig;

let config = AppConfig::load()?;

// Database path is resolved relative to data_dir
let db_path = config.db_path();  // "data/forge.db"

// Blob storage uses data_dir
let store = Store::from_config(
    db_path,
    &config.storage.data_dir,
).await?;
```

## Migrations

Database migrations are managed automatically using sqlx migrations. Migration files are located in `crates/forge_store/migrations/` and are applied automatically when the store is initialized.

### Migration Files

```
migrations/
├── 20250101000001_create_models.sql
├── 20250101000002_create_sessions.sql
├── 20250101000003_create_messages.sql
├── 20250101000004_create_jobs.sql
└── 20250101000005_create_documents.sql
```

Migrations run automatically on:
- `Store::new_in_memory()`
- `Store::new_with_file()`
- `Store::from_config()`

## Testing

The store includes comprehensive tests:

```bash
# Run all store tests
cargo test -p forge_store

# Run specific test
cargo test -p forge_store test_model_crud
```

Test coverage includes:
- Database initialization and migrations
- Full CRUD operations for all entities
- Blob storage operations
- Integration tests (database + blob storage)
- Cascade deletes (sessions -> messages)
- Job lifecycle management

## Best Practices

1. **Use repositories**: Access data through repository methods, not raw SQL
2. **Close connections**: Call `store.close().await` when done
3. **Use transactions**: For atomic operations across multiple entities
4. **Validate data**: Use entity constructors and validation methods
5. **Handle errors**: All operations return `Result<T, StoreError>`
6. **Use WAL mode**: Enables concurrent reads and writes
7. **Blob references**: Store blob paths in database records
8. **Cleanup**: Delete blobs when deleting database records

## Performance Considerations

- **Connection pool**: Configured via `db.pool_size` (default: 5)
- **WAL mode**: Better concurrency for reads/writes
- **Indexes**: All foreign keys and common query fields are indexed
- **Prepared statements**: sqlx uses prepared statements automatically
- **Blob chunking**: For very large files, consider streaming APIs

## Future Enhancements

- Vector embeddings table for RAG
- Full-text search on documents
- Blob compression
- Incremental backups
- Replication support
- S3/cloud blob storage backend
