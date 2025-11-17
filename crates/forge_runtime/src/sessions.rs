//! # Session Management
//!
//! Database-backed session and message management.

use chrono::{DateTime, Utc};
use forge_store::{Message as DbMessage, Session as DbSession, Store};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info};

/// Errors that can occur during session operations.
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    Storage(#[from] forge_store::StoreError),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

pub type Result<T> = std::result::Result<T, SessionError>;

/// Role of a message sender.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl From<MessageRole> for String {
    fn from(role: MessageRole) -> String {
        match role {
            MessageRole::User => "user".to_string(),
            MessageRole::Assistant => "assistant".to_string(),
            MessageRole::System => "system".to_string(),
        }
    }
}

impl std::str::FromStr for MessageRole {
    type Err = SessionError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            "system" => Ok(MessageRole::System),
            _ => Err(SessionError::InvalidMessage(format!("Unknown role: {}", s))),
        }
    }
}

/// A message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Message {
    /// Convert from database message.
    pub fn from_db(msg: DbMessage) -> Result<Self> {
        let role = msg.role.parse()?;
        let created_at = DateTime::parse_from_rfc3339(&msg.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| SessionError::InvalidMessage(format!("Invalid timestamp: {}", e)))?;

        Ok(Self {
            id: msg.id,
            session_id: msg.session_id,
            role,
            content: msg.content,
            created_at,
        })
    }

    /// Convert to database message.
    pub fn to_db(&self) -> DbMessage {
        DbMessage {
            id: self.id.clone(),
            session_id: self.session_id.clone(),
            role: self.role.clone().into(),
            content: self.content.clone(),
            meta_json: None,
            created_at: self.created_at.to_rfc3339(),
        }
    }
}

/// A chat session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub model_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
}

impl Session {
    /// Convert from database session.
    pub fn from_db(session: DbSession, messages: Vec<DbMessage>) -> Result<Self> {
        let created_at = DateTime::parse_from_rfc3339(&session.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| SessionError::InvalidMessage(format!("Invalid timestamp: {}", e)))?;

        let updated_at = DateTime::parse_from_rfc3339(&session.updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| SessionError::InvalidMessage(format!("Invalid timestamp: {}", e)))?;

        let messages: Result<Vec<Message>> = messages
            .into_iter()
            .map(Message::from_db)
            .collect();

        Ok(Self {
            id: session.id,
            name: session.name,
            model_id: session.model_id,
            created_at,
            updated_at,
            messages: messages?,
        })
    }
}

/// Manager for chat sessions with database persistence.
pub struct SessionManager {
    store: Arc<Store>,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    /// Create a new session.
    pub async fn create_session(&self, name: String, model_id: Option<String>) -> Result<Session> {
        info!("Creating new session: {}", name);

        let db_session = DbSession::new(name, model_id);
        self.store.sessions().create(&db_session).await?;

        debug!("Session created: {}", db_session.id);

        Session::from_db(db_session, vec![])
    }

    /// Get a session by ID, including its messages.
    pub async fn get_session(&self, id: &str) -> Result<Session> {
        debug!("Getting session: {}", id);

        let db_session = self.store.sessions().get(id).await
            .map_err(|_| SessionError::NotFound(id.to_string()))?;

        let messages = self.store.messages().get_by_session(id).await?;

        Session::from_db(db_session, messages)
    }

    /// List all sessions (without messages for efficiency).
    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        debug!("Listing all sessions");

        let db_sessions = self.store.sessions().list().await?;

        let sessions: Result<Vec<Session>> = db_sessions
            .into_iter()
            .map(|s| Session::from_db(s, vec![]))
            .collect();

        sessions
    }

    /// Add a message to a session.
    pub async fn add_message(&self, session_id: &str, role: MessageRole, content: String) -> Result<Message> {
        debug!("Adding {} message to session: {}", match role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
        }, session_id);

        // Verify session exists
        self.store.sessions().get(session_id).await
            .map_err(|_| SessionError::NotFound(session_id.to_string()))?;

        // Create message
        let db_message = DbMessage::new(session_id.to_string(), role.into(), content);
        self.store.messages().create(&db_message).await?;

        // Update session timestamp
        let mut session = self.store.sessions().get(session_id).await?;
        session.updated_at = Utc::now().to_rfc3339();
        self.store.sessions().update(&session).await?;

        Message::from_db(db_message)
    }

    /// Get messages for a session.
    pub async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>> {
        debug!("Getting messages for session: {}", session_id);

        let db_messages = self.store.messages().get_by_session(session_id).await?;

        let messages: Result<Vec<Message>> = db_messages
            .into_iter()
            .map(Message::from_db)
            .collect();

        messages
    }

    /// Delete a session and all its messages.
    pub async fn delete_session(&self, id: &str) -> Result<()> {
        info!("Deleting session: {}", id);

        // Messages will be cascade deleted by foreign key constraint
        self.store.sessions().delete(id).await
            .map_err(|_| SessionError::NotFound(id.to_string()))?;

        Ok(())
    }

    /// Update session configuration (e.g., model_id).
    pub async fn update_session(&self, id: &str, model_id: Option<String>) -> Result<()> {
        debug!("Updating session: {}", id);

        let mut session = self.store.sessions().get(id).await
            .map_err(|_| SessionError::NotFound(id.to_string()))?;

        session.model_id = model_id;
        session.updated_at = Utc::now().to_rfc3339();

        self.store.sessions().update(&session).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_manager() -> SessionManager {
        let store = Arc::new(Store::new_in_memory().await.unwrap());
        SessionManager::new(store)
    }

    #[tokio::test]
    async fn test_create_session() {
        let manager = setup_manager().await;

        let session = manager.create_session("Test Session".to_string(), None).await.unwrap();

        assert_eq!(session.name, "Test Session");
        assert_eq!(session.messages.len(), 0);
        assert!(session.model_id.is_none());
    }

    #[tokio::test]
    async fn test_get_session() {
        let manager = setup_manager().await;

        let created = manager.create_session("Test".to_string(), None).await.unwrap();
        let retrieved = manager.get_session(&created.id).await.unwrap();

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Test");
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let manager = setup_manager().await;

        manager.create_session("Session 1".to_string(), None).await.unwrap();
        manager.create_session("Session 2".to_string(), None).await.unwrap();

        let sessions = manager.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_add_message() {
        let manager = setup_manager().await;

        let session = manager.create_session("Test".to_string(), None).await.unwrap();

        let message = manager.add_message(
            &session.id,
            MessageRole::User,
            "Hello".to_string(),
        ).await.unwrap();

        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, "Hello");

        let messages = manager.get_messages(&session.id).await.unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_add_multiple_messages() {
        let manager = setup_manager().await;

        let session = manager.create_session("Test".to_string(), None).await.unwrap();

        manager.add_message(&session.id, MessageRole::User, "Hello".to_string()).await.unwrap();
        manager.add_message(&session.id, MessageRole::Assistant, "Hi there!".to_string()).await.unwrap();
        manager.add_message(&session.id, MessageRole::User, "How are you?".to_string()).await.unwrap();

        let messages = manager.get_messages(&session.id).await.unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, MessageRole::User);
        assert_eq!(messages[1].role, MessageRole::Assistant);
        assert_eq!(messages[2].role, MessageRole::User);
    }

    #[tokio::test]
    #[ignore] // TODO: Fix test - schema mismatch with config_json field
    async fn test_update_session() {
        let manager = setup_manager().await;

        let session = manager.create_session("Test".to_string(), None).await.unwrap();

        manager.update_session(&session.id, Some("model-123".to_string())).await.unwrap();

        let updated = manager.get_session(&session.id).await.unwrap();
        assert_eq!(updated.model_id, Some("model-123".to_string()));
    }

    #[tokio::test]
    async fn test_delete_session() {
        let manager = setup_manager().await;

        let session = manager.create_session("Test".to_string(), None).await.unwrap();
        manager.add_message(&session.id, MessageRole::User, "Test".to_string()).await.unwrap();

        manager.delete_session(&session.id).await.unwrap();

        assert!(manager.get_session(&session.id).await.is_err());
    }

    #[tokio::test]
    async fn test_session_not_found() {
        let manager = setup_manager().await;

        let result = manager.get_session("nonexistent").await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }
}
