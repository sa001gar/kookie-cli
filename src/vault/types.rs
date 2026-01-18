//! Secret types for the vault

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All supported secret types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SecretType {
    Password,
    ApiKey,
    Note,
    DbCredential,
    Token,
}

impl std::fmt::Display for SecretType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretType::Password => write!(f, "password"),
            SecretType::ApiKey => write!(f, "api-key"),
            SecretType::Note => write!(f, "note"),
            SecretType::DbCredential => write!(f, "db-credential"),
            SecretType::Token => write!(f, "token"),
        }
    }
}

/// Password secret
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Password {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Password {
    pub fn new(
        name: String,
        password: String,
        description: Option<String>,
        username: Option<String>,
        url: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            username,
            password,
            url,
            created_at: now,
            updated_at: now,
        }
    }
}

/// API Key secret
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub key: String,
    pub service: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ApiKey {
    pub fn new(
        name: String,
        key: String,
        description: Option<String>,
        service: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            key,
            service,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Private note secret
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Note {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new(name: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            content,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Database credential secret
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DbCredential {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub host: String,
    pub port: Option<u16>,
    pub database: String,
    pub username: String,
    pub password: String,
    pub db_type: Option<String>, // postgres, mysql, mongodb, etc.
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DbCredential {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        host: String,
        port: Option<u16>,
        database: String,
        username: String,
        password: String,
        db_type: Option<String>,
        description: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            host,
            port,
            database,
            username,
            password,
            db_type,
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns a connection string for the database
    pub fn connection_string(&self) -> String {
        let db_type = self.db_type.as_deref().unwrap_or("postgres");
        let port = self.port.unwrap_or(match db_type {
            "postgres" | "postgresql" => 5432,
            "mysql" => 3306,
            "mongodb" => 27017,
            _ => 5432,
        });
        
        match db_type {
            "mongodb" => format!(
                "mongodb://{}:{}@{}:{}/{}",
                self.username, self.password, self.host, port, self.database
            ),
            _ => format!(
                "{}://{}:{}@{}:{}/{}",
                db_type, self.username, self.password, self.host, port, self.database
            ),
        }
    }
}

/// Token secret (JWT, OAuth, etc.)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Token {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub token: String,
    pub token_type: Option<String>, // jwt, oauth, bearer, etc.
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Token {
    pub fn new(
        name: String,
        token: String,
        description: Option<String>,
        token_type: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            token,
            token_type,
            expires_at,
            created_at: now,
            updated_at: now,
        }
    }

    /// Checks if the token is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| exp < Utc::now()).unwrap_or(false)
    }
}
