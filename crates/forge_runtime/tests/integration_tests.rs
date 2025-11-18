//! Integration tests for forge_runtime

use forge_engine::{EchoBackend, Engine};
use forge_models::ModelRegistry;
use forge_runtime::{Config, Runtime};
use forge_store::Store;
use std::sync::Arc;

#[tokio::test]
async fn test_runtime_initialization() {
    let store = Store::new_in_memory().await.unwrap();
    let backend = Arc::new(EchoBackend::new());
    let engine = Arc::new(Engine::new(backend));
    let registry = Arc::new(ModelRegistry::new(store.clone()));

    let config = Config::default();
    let runtime = Runtime::new(config, store, engine, registry);

    assert!(runtime.get_stats().await.is_ok());
}

#[tokio::test]
async fn test_session_lifecycle() {
    let store = Store::new_in_memory().await.unwrap();
    let backend = Arc::new(EchoBackend::new());
    let engine = Arc::new(Engine::new(backend));
    let registry = Arc::new(ModelRegistry::new(store.clone()));

    let config = Config::default();
    let runtime = Runtime::new(config, store.clone(), engine, registry);

    // Create a session
    let session_name = "test-session";
    let session = runtime
        .create_session(session_name.to_string(), None)
        .await
        .unwrap();

    assert_eq!(session.name, session_name);

    // Get the session
    let retrieved = runtime.get_session(&session.id).await.unwrap();
    assert_eq!(retrieved.id, session.id);

    // List sessions
    let sessions = runtime.list_sessions().await.unwrap();
    assert!(sessions.iter().any(|s| s.id == session.id));
}

#[tokio::test]
async fn test_rag_document_lifecycle() {
    let store = Store::new_in_memory().await.unwrap();
    let backend = Arc::new(EchoBackend::new());
    let engine = Arc::new(Engine::new(backend));
    let registry = Arc::new(ModelRegistry::new(store.clone()));

    let config = Config::default();
    let runtime = Runtime::new(config, store.clone(), engine, registry);

    // Add a document
    let doc_id = runtime
        .add_document(
            "test-doc.txt".to_string(),
            "This is a test document for RAG.".to_string(),
        )
        .await
        .unwrap();

    // Get the document
    let doc = runtime.get_document(&doc_id).await.unwrap();
    assert_eq!(doc.source, "test-doc.txt");

    // List documents
    let docs = runtime.list_documents().await.unwrap();
    assert!(docs.iter().any(|d| d.id == doc_id));

    // Delete document
    runtime.delete_document(&doc_id).await.unwrap();
    let result = runtime.get_document(&doc_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_training_job_creation() {
    let store = Store::new_in_memory().await.unwrap();
    let backend = Arc::new(EchoBackend::new());
    let engine = Arc::new(Engine::new(backend));
    let registry = Arc::new(ModelRegistry::new(store.clone()));

    let config = Config::default();
    let runtime = Runtime::new(config, store.clone(), engine, registry);

    // Create a training job
    let job = runtime
        .create_training_job(
            "test-model".to_string(),
            "dataset.jsonl".to_string(),
            serde_json::json!({"epochs": 3}),
        )
        .await
        .unwrap();

    assert_eq!(job.kind, "training");
    assert_eq!(job.status, "queued");

    // Get the job
    let retrieved = runtime.get_job(&job.id).await.unwrap();
    assert_eq!(retrieved.id, job.id);

    // List jobs
    let jobs = runtime.list_jobs().await.unwrap();
    assert!(jobs.iter().any(|j| j.id == job.id));
}

#[tokio::test]
async fn test_runtime_stats() {
    let store = Store::new_in_memory().await.unwrap();
    let backend = Arc::new(EchoBackend::new());
    let engine = Arc::new(Engine::new(backend));
    let registry = Arc::new(ModelRegistry::new(store.clone()));

    let config = Config::default();
    let runtime = Runtime::new(config, store.clone(), engine, registry);

    let stats = runtime.get_stats().await.unwrap();

    // Stats should have basic runtime information
    assert!(stats.contains_key("uptime_seconds"));
    assert!(stats.contains_key("models_loaded"));
}

#[tokio::test]
async fn test_concurrent_session_operations() {
    let store = Store::new_in_memory().await.unwrap();
    let backend = Arc::new(EchoBackend::new());
    let engine = Arc::new(Engine::new(backend));
    let registry = Arc::new(ModelRegistry::new(store.clone()));

    let config = Config::default();
    let runtime = Arc::new(Runtime::new(config, store.clone(), engine, registry));

    // Create multiple sessions concurrently
    let mut handles = vec![];
    for i in 0..5 {
        let runtime_clone = Arc::clone(&runtime);
        let handle = tokio::spawn(async move {
            runtime_clone
                .create_session(format!("session-{}", i), None)
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Verify all sessions were created
    let sessions = runtime.list_sessions().await.unwrap();
    assert_eq!(sessions.len(), 5);
}
