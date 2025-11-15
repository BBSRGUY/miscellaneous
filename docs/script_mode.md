# Script Mode - Data Processing and Job Orchestration

A comprehensive guide to building data pipelines and orchestrating workflows with Blang's script mode.

## Table of Contents

- [Introduction](#introduction)
- [Script Basics](#script-basics)
- [Configuration](#configuration)
- [Jobs](#jobs)
- [Steps](#steps)
- [Parallel Execution](#parallel-execution)
- [Dependencies](#dependencies)
- [Error Handling](#error-handling)
- [Timeouts and Retries](#timeouts-and-retries)
- [Data Flow](#data-flow)
- [Accessing Results](#accessing-results)
- [Conditionals and Control Flow](#conditionals-and-control-flow)
- [Best Practices](#best-practices)

## Introduction

Script mode is Blang's approach to building data processing pipelines and orchestrating complex workflows. Inspired by REXX's job control, script mode provides declarative syntax for defining jobs, steps, and their dependencies.

### Key Concepts

- **Scripts**: Top-level containers for workflows
- **Jobs**: Logical units of work that can run independently or depend on other jobs
- **Steps**: Individual operations within a job
- **Parallelism**: Execute multiple steps or jobs concurrently
- **Dependencies**: Define execution order with `depends_on`
- **Resilience**: Built-in retry, timeout, and error handling

## Script Basics

### Minimal Script

```blang
script HelloWorld {
    job greet {
        step say_hello {
            println("Hello from script mode!");
        }
    }
}
```

### Script Structure

A script can have these sections:

```blang
script MyPipeline {
    // Optional: Configuration
    config {
        max_parallel: 4,
        timeout: 300s,
        on_error: "continue",
    }

    // Jobs define units of work
    job extract {
        step fetch_data {
            // Step implementation
        }
    }

    job transform depends_on(extract) {
        step process_data {
            // Step implementation
        }
    }

    // Helper functions (optional)
    fn validate_data(data: Data) -> bool {
        // Validation logic
    }
}
```

## Configuration

### Global Configuration

Configure script-wide settings:

```blang
script DataPipeline {
    config {
        // Maximum parallel jobs
        max_parallel: 4,

        // Global timeout for entire script
        timeout: 600s,

        // Error handling strategy
        on_error: "stop",  // "stop", "continue", or "retry"

        // Retry configuration
        max_retries: 3,
        retry_delay: 2s,

        // Logging level
        log_level: "info",  // "debug", "info", "warn", "error"
    }
}
```

### Job-Level Configuration

Override global settings per job:

```blang
script Pipeline {
    config {
        max_parallel: 4,
        timeout: 300s,
    }

    job fetch_data {
        config {
            timeout: 60s,
            max_retries: 5,
        }

        step get_api_data {
            return http.get("/api/data");
        }
    }
}
```

## Jobs

### Basic Job

```blang
script Example {
    job process {
        step load {
            let data = read_file("data.json");
            return parse_json(data);
        }

        step transform {
            return process(load.result);
        }
    }
}
```

### Multiple Jobs

```blang
script ETL {
    job extract {
        step fetch {
            return http.get("/api/users");
        }
    }

    job transform {
        step clean {
            let raw = extract.fetch.result;
            return sanitize(raw);
        }
    }

    job load {
        step save {
            let clean_data = transform.clean.result;
            db.insert("users", clean_data);
        }
    }
}
```

## Steps

### Step Basics

Steps are the fundamental units of work:

```blang
script Example {
    job process {
        step simple {
            // Direct execution
            println("Processing...");
        }

        step with_return {
            // Return a value
            return 42;
        }

        step with_variables {
            // Local variables
            let x = 10;
            let y = 20;
            return x + y;
        }
    }
}
```

### Step Configuration

Configure individual steps:

```blang
script Resilient {
    job fetch {
        step get_data {
            retry: 3,
            timeout: 30s,
            on_error: "retry",

            return http.get("/api/data");
        }

        step process_data {
            timeout: 60s,
            on_error: "stop",

            let data = get_data.result;
            return transform(data);
        }
    }
}
```

## Parallel Execution

### Parallel Steps

Execute multiple steps concurrently:

```blang
script ParallelFetch {
    job gather_data {
        parallel {
            step fetch_users {
                return http.get("/api/users");
            }

            step fetch_products {
                return http.get("/api/products");
            }

            step fetch_orders {
                return http.get("/api/orders");
            }
        }

        // This step runs after all parallel steps complete
        step combine {
            let users = fetch_users.result;
            let products = fetch_products.result;
            let orders = fetch_orders.result;

            return merge_data(users, products, orders);
        }
    }
}
```

### Parallel Jobs

Multiple jobs can run in parallel if they have no dependencies:

```blang
script MultiSource {
    // These jobs run in parallel (no dependencies)
    job fetch_api_a {
        step get {
            return http.get("https://api-a.example.com/data");
        }
    }

    job fetch_api_b {
        step get {
            return http.get("https://api-b.example.com/data");
        }
    }

    job fetch_api_c {
        step get {
            return http.get("https://api-c.example.com/data");
        }
    }

    // This job waits for all above jobs
    job combine depends_on(fetch_api_a, fetch_api_b, fetch_api_c) {
        step merge {
            return combine_results([
                fetch_api_a.get.result,
                fetch_api_b.get.result,
                fetch_api_c.get.result,
            ]);
        }
    }
}
```

## Dependencies

### Simple Dependencies

Use `depends_on` to define execution order:

```blang
script Pipeline {
    job step1 {
        step run {
            println("Running step 1");
        }
    }

    job step2 depends_on(step1) {
        step run {
            println("Running step 2 after step 1");
        }
    }

    job step3 depends_on(step2) {
        step run {
            println("Running step 3 after step 2");
        }
    }
}
```

### Multiple Dependencies

A job can depend on multiple jobs:

```blang
script ComplexPipeline {
    job fetch_users {
        step get {
            return http.get("/api/users");
        }
    }

    job fetch_orders {
        step get {
            return http.get("/api/orders");
        }
    }

    // Waits for both fetch_users and fetch_orders
    job join_data depends_on(fetch_users, fetch_orders) {
        step merge {
            let users = fetch_users.get.result;
            let orders = fetch_orders.get.result;
            return join(users, orders, "user_id");
        }
    }

    job report depends_on(join_data) {
        step generate {
            let data = join_data.merge.result;
            return create_report(data);
        }
    }
}
```

### Diamond Dependencies

Handle complex dependency graphs:

```blang
script DiamondPattern {
    job source {
        step load {
            return load_data("input.csv");
        }
    }

    job branch_a depends_on(source) {
        step process {
            return process_a(source.load.result);
        }
    }

    job branch_b depends_on(source) {
        step process {
            return process_b(source.load.result);
        }
    }

    // Waits for both branches
    job merge depends_on(branch_a, branch_b) {
        step combine {
            return combine(
                branch_a.process.result,
                branch_b.process.result
            );
        }
    }
}
```

## Error Handling

### Try-Catch in Steps

```blang
script ErrorHandling {
    job robust_fetch {
        step get_data {
            retry: 3,
            on_error: "continue",

            try {
                return http.get("/api/data");
            } catch (HttpError e) {
                log.error("HTTP request failed: " + e.message);
                return default_data();
            } catch (TimeoutError e) {
                log.error("Request timed out: " + e.message);
                return cached_data();
            }
        }

        step process {
            if get_data.error {
                log.warn("Using fallback data");
            }

            let data = get_data.result;
            return transform(data);
        }
    }
}
```

### Job-Level Error Handling

```blang
script PipelineWithFallback {
    job primary {
        config {
            on_error: "stop",
        }

        step fetch {
            return http.get("/api/primary");
        }
    }

    job fallback depends_on(primary) {
        // Only runs if primary succeeds
        step use_primary {
            if !primary.error {
                return primary.fetch.result;
            } else {
                return http.get("/api/fallback");
            }
        }
    }
}
```

## Timeouts and Retries

### Step-Level Timeouts

```blang
script TimeoutExample {
    job fetch {
        step quick_operation {
            timeout: 5s,

            return fast_api_call();
        }

        step slow_operation {
            timeout: 120s,

            return expensive_computation();
        }
    }
}
```

### Retry Configuration

```blang
script RetryExample {
    job resilient_fetch {
        step get_data {
            retry: 5,
            retry_delay: 2s,
            retry_backoff: "exponential",  // "fixed", "linear", "exponential"
            timeout: 30s,

            return http.get("/api/unreliable-endpoint");
        }

        step process {
            retry: 3,
            retry_delay: 1s,
            retry_backoff: "linear",

            return process_data(get_data.result);
        }
    }
}
```

### Conditional Retry

```blang
script SmartRetry {
    job fetch {
        step get_with_conditional_retry {
            retry: 5,
            timeout: 30s,

            try {
                let response = http.get("/api/data");

                if response.status == 429 {
                    // Rate limited - wait and retry
                    sleep(5s);
                    return Err("Rate limited, retrying...");
                } else if response.status >= 500 {
                    // Server error - retry
                    return Err("Server error, retrying...");
                } else if response.status >= 400 {
                    // Client error - don't retry
                    return Err("Client error, not retrying");
                } else {
                    return Ok(response.data);
                }
            } catch (NetworkError e) {
                // Network error - retry
                return Err("Network error: " + e.message);
            }
        }
    }
}
```

## Data Flow

### Passing Data Between Steps

```blang
script DataFlow {
    job pipeline {
        step load {
            return read_file("data.json");
        }

        step parse {
            let raw = load.result;
            return json.parse(raw);
        }

        step transform {
            let data = parse.result;
            return data.map(|item| {
                {
                    id: item.id,
                    name: item.name.to_uppercase(),
                    timestamp: now(),
                }
            });
        }

        step filter {
            let items = transform.result;
            return items.filter(|item| item.id > 100);
        }

        step save {
            let final_data = filter.result;
            write_file("output.json", json.stringify(final_data));
        }
    }
}
```

### Sharing Data Across Jobs

```blang
script CrossJobData {
    job extract {
        step fetch {
            return http.get("/api/data");
        }

        step validate {
            let data = fetch.result;
            return data.filter(|item| item.valid);
        }
    }

    job transform depends_on(extract) {
        step enrich {
            // Access data from previous job
            let validated = extract.validate.result;

            return validated.map(|item| {
                let extra = fetch_extra_info(item.id);
                return merge(item, extra);
            });
        }
    }

    job load depends_on(transform) {
        step save {
            // Access data from transform job
            let enriched = transform.enrich.result;
            db.insert("processed_data", enriched);
        }
    }
}
```

## Accessing Results

### Result Access Patterns

```blang
script ResultAccess {
    job example {
        step step1 {
            return 42;
        }

        step step2 {
            // Access previous step result
            let value = step1.result;
            return value * 2;
        }

        step step3 {
            // Access multiple step results
            let a = step1.result;
            let b = step2.result;
            return a + b;
        }
    }

    job another depends_on(example) {
        step use_previous_job {
            // Access results from completed job
            let result = example.step3.result;
            return result + 100;
        }
    }
}
```

### Checking Step Status

```blang
script StatusChecking {
    job monitor {
        step risky_operation {
            retry: 3,
            on_error: "continue",

            return unstable_api_call();
        }

        step handle_result {
            if risky_operation.success {
                let data = risky_operation.result;
                return process(data);
            } else if risky_operation.error {
                log.error("Operation failed: " + risky_operation.error_message);
                return fallback_data();
            }
        }
    }
}
```

## Conditionals and Control Flow

### Conditional Steps

```blang
script ConditionalExecution {
    job smart_process {
        step check_cache {
            return cache.get("data_key");
        }

        step fetch_if_needed {
            if check_cache.result.is_none() {
                return http.get("/api/data");
            } else {
                return check_cache.result.unwrap();
            }
        }

        step process {
            let data = fetch_if_needed.result;
            return transform(data);
        }
    }
}
```

### Conditional Jobs

```blang
script ConditionalJobs {
    job check {
        step determine {
            return should_run_intensive_job();
        }
    }

    job intensive depends_on(check) {
        step run {
            if check.determine.result {
                return expensive_computation();
            } else {
                log.info("Skipping intensive job");
                return None;
            }
        }
    }
}
```

## Best Practices

### 1. Organize Jobs Logically

```blang
// Good: Clear separation of concerns
script ETLPipeline {
    job extract {
        step fetch_source_a { /* ... */ }
        step fetch_source_b { /* ... */ }
    }

    job transform depends_on(extract) {
        step clean { /* ... */ }
        step normalize { /* ... */ }
        step enrich { /* ... */ }
    }

    job load depends_on(transform) {
        step save_to_db { /* ... */ }
        step update_cache { /* ... */ }
    }
}
```

### 2. Use Meaningful Names

```blang
// Good: Descriptive names
script UserDataSync {
    job fetch_active_users {
        step query_database {
            return db.query("SELECT * FROM users WHERE active = true");
        }
    }
}

// Bad: Vague names
script Process {
    job job1 {
        step step1 {
            return db.query("SELECT * FROM users WHERE active = true");
        }
    }
}
```

### 3. Handle Errors Gracefully

```blang
script RobustPipeline {
    config {
        on_error: "continue",
        log_level: "info",
    }

    job fetch {
        step get_data {
            retry: 3,
            retry_delay: 2s,
            timeout: 30s,

            try {
                return http.get("/api/data");
            } catch (HttpError e) {
                log.error("Fetch failed: " + e.message);
                return fallback_data();
            }
        }
    }
}
```

### 4. Leverage Parallelism

```blang
// Good: Independent operations run in parallel
script EfficientFetch {
    job gather {
        parallel {
            step fetch_a { return http.get("/api/a"); }
            step fetch_b { return http.get("/api/b"); }
            step fetch_c { return http.get("/api/c"); }
        }

        step combine {
            return merge([
                fetch_a.result,
                fetch_b.result,
                fetch_c.result,
            ]);
        }
    }
}
```

### 5. Keep Steps Focused

```blang
// Good: Each step does one thing
script Focused {
    job process {
        step load {
            return read_file("data.csv");
        }

        step parse {
            return csv.parse(load.result);
        }

        step validate {
            return validate_schema(parse.result);
        }

        step transform {
            return apply_transformations(validate.result);
        }
    }
}

// Bad: Monolithic step
script Monolithic {
    job process {
        step do_everything {
            let data = read_file("data.csv");
            let parsed = csv.parse(data);
            let validated = validate_schema(parsed);
            return apply_transformations(validated);
        }
    }
}
```

### 6. Use Configuration Wisely

```blang
script Configured {
    config {
        // Global defaults
        max_parallel: 4,
        timeout: 300s,
        max_retries: 3,
    }

    job critical {
        config {
            // Override for critical job
            timeout: 600s,
            max_retries: 5,
        }

        step important {
            // Override for important step
            retry: 10,
            timeout: 120s,

            return critical_operation();
        }
    }
}
```

### 7. Document Complex Workflows

```blang
script ComplexPipeline {
    // This pipeline processes user data from multiple sources
    // 1. Extract: Fetch from API and database
    // 2. Transform: Clean, normalize, and enrich
    // 3. Load: Save to data warehouse and update cache

    job extract {
        // Fetch user data from all sources in parallel
        parallel {
            step fetch_api {
                return http.get("/api/users");
            }

            step fetch_db {
                return db.query("SELECT * FROM legacy_users");
            }
        }
    }

    job transform depends_on(extract) {
        // Clean and normalize data
        step clean {
            let api_data = extract.fetch_api.result;
            let db_data = extract.fetch_db.result;
            return sanitize(merge(api_data, db_data));
        }

        // Enrich with additional information
        step enrich {
            return add_metadata(clean.result);
        }
    }

    job load depends_on(transform) {
        // Save to warehouse and update cache
        parallel {
            step save_warehouse {
                warehouse.insert("users", transform.enrich.result);
            }

            step update_cache {
                cache.set("users", transform.enrich.result);
            }
        }
    }
}
```

---

For more information, see:

- [Language Overview](language_overview.md) - Core language features
- [Component Mode](component_mode.md) - Building reactive UI components
- [Unsafe Blocks](unsafe_blocks.md) - Low-level programming
