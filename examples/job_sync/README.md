# Job Sync Example

A data synchronization pipeline demonstrating Blang's script mode for orchestrating workflows.

## What This Example Demonstrates

- Script mode syntax and structure
- Job definitions and dependencies
- Parallel step execution
- Error handling and retries
- Timeouts and configuration
- Data flow between jobs and steps
- ETL (Extract, Transform, Load) pattern

## The Code

This example implements a data synchronization pipeline that:

1. **Extract**: Fetches data from multiple sources in parallel
   - User data from API
   - Order data from database
   - Product data from external service

2. **Transform**: Processes and enriches the data
   - Validates data schemas
   - Merges related data
   - Enriches with additional information
   - Normalizes formats

3. **Load**: Saves the processed data
   - Writes to data warehouse
   - Updates cache
   - Triggers notifications

## File Structure

```
job_sync/
├── README.md          # This file
└── main.bs            # Data sync pipeline (script mode)
```

## Features

- ✅ Parallel data fetching from multiple sources
- ✅ Automatic retry on failures
- ✅ Configurable timeouts
- ✅ Error handling with fallbacks
- ✅ Data validation and transformation
- ✅ Dependency management between jobs
- ✅ Progress logging

## Running the Example

### Using the CLI

```bash
# Navigate to this directory
cd examples/job_sync

# Run the script
blang run main.bs

# Or compile and run
blang compile main.bs -o sync.wasm
blang execute sync.wasm
```

### Configuration

You can configure the pipeline through environment variables or config file:

```bash
# Set API endpoints
export API_USERS_URL="https://api.example.com/users"
export API_ORDERS_URL="https://api.example.com/orders"
export API_PRODUCTS_URL="https://api.example.com/products"

# Run with configuration
blang run main.bs
```

## Understanding the Code

### Script Structure

```blang
script DataSync {
    config {
        max_parallel: 4,
        timeout: 300s,
    }

    job extract { /* ... */ }
    job transform depends_on(extract) { /* ... */ }
    job load depends_on(transform) { /* ... */ }
}
```

Scripts organize workflows into jobs with dependencies.

### Parallel Execution

```blang
job extract {
    parallel {
        step fetch_users { /* ... */ }
        step fetch_orders { /* ... */ }
        step fetch_products { /* ... */ }
    }
}
```

Independent steps run concurrently for better performance.

### Dependencies

```blang
job transform depends_on(extract) {
    // This job waits for extract to complete
}

job load depends_on(transform) {
    // This job waits for transform to complete
}
```

Jobs run in order based on their dependencies.

### Error Handling

```blang
step fetch_data {
    retry: 3,
    retry_delay: 2s,
    timeout: 30s,

    try {
        return http.get("/api/data");
    } catch (HttpError e) {
        return fallback_data();
    }
}
```

Configure retries, timeouts, and fallback behavior.

### Data Flow

```blang
job transform depends_on(extract) {
    step merge {
        // Access results from previous job
        let users = extract.fetch_users.result;
        let orders = extract.fetch_orders.result;
        return join(users, orders);
    }
}
```

Access results from previous jobs and steps.

## Pipeline Flow

```
┌─────────────────────────────────────────┐
│           Extract (Parallel)            │
├──────────────┬───────────────┬──────────┤
│ Fetch Users  │ Fetch Orders  │ Products │
└──────┬───────┴───────┬───────┴────┬─────┘
       │               │            │
       └───────────────┴────────────┘
                       │
       ┌───────────────▼────────────────┐
       │          Transform              │
       ├────────────┬────────────────────┤
       │ Validate   │ Merge   │ Enrich   │
       └────────┬───┴─────┬───┴────┬─────┘
                │         │        │
                └─────────┴────────┘
                          │
         ┌────────────────▼───────────────┐
         │             Load                │
         ├──────────────┬─────────────────┤
         │  Warehouse   │  Cache  │ Notify │
         └──────────────┴─────────────────┘
```

## Code Highlights

### Retry Logic

```blang
step fetch_users {
    retry: 5,
    retry_delay: 2s,
    retry_backoff: "exponential",
    timeout: 30s,

    return http.get(config.users_api);
}
```

Automatically retries on failure with exponential backoff.

### Data Validation

```blang
step validate {
    let raw_data = merge.result;

    for item in raw_data {
        if !is_valid_schema(item) {
            log.warn("Invalid item: " + item.id);
        }
    }

    return raw_data.filter(is_valid_schema);
}
```

Validates data before processing.

### Conditional Execution

```blang
step save_to_warehouse {
    if enrich.result.len() > 0 {
        warehouse.batch_insert("sync_data", enrich.result);
        log.info("Saved " + enrich.result.len() + " records");
    } else {
        log.warn("No data to save");
    }
}
```

Execute steps conditionally based on data.

## Next Steps

After understanding this example, explore:

- **[docs/script_mode.md](../../docs/script_mode.md)**: Complete script mode documentation
- **[docs/language_overview.md](../../docs/language_overview.md)**: Full language reference
- **[counter_app](../counter_app/README.md)**: Component mode basics

## Extending This Example

Ideas for extending this pipeline:

1. **Incremental Sync**: Only fetch changed data since last run
2. **Checkpointing**: Resume from last successful step on failure
3. **Monitoring**: Send metrics to monitoring system
4. **Scheduling**: Run on a schedule (cron-like)
5. **Webhooks**: Trigger pipeline from external events
6. **Data Quality**: Add data quality checks and alerts
7. **Multiple Destinations**: Load to multiple targets
8. **Rollback**: Implement rollback on failure

## Performance Tips

1. **Use parallelism**: Fetch independent data sources in parallel
2. **Configure timeouts**: Set appropriate timeouts for each step
3. **Batch operations**: Process data in batches for efficiency
4. **Cache results**: Cache frequently accessed data
5. **Monitor metrics**: Track pipeline performance and bottlenecks

## Learning Resources

- [Script Mode Guide](../../docs/script_mode.md) - Complete script mode documentation
- [Language Overview](../../docs/language_overview.md) - Core Blang syntax
- [Todo App](../todo_app/README.md) - Component mode example
