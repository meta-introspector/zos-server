#![allow(unused)]

use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub permissions: Vec<String>,
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.username.as_bytes()
    }
}

#[derive(Clone)]
pub struct Backend {
    users: Arc<RwLock<HashMap<String, User>>>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_user(&self, username: String) -> User {
        let user = User {
            id: rand::random(),
            username: username.clone(),
            permissions: if username == "root" {
                vec!["admin".to_string(), "dashboard".to_string()]
            } else {
                vec!["dashboard".to_string()]
            },
        };

        self.users.write().await.insert(username, user.clone());
        user
    }
}

#[async_trait::async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = String; // username
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let users = self.users.read().await;
        Ok(users.get(&creds).cloned())
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let users = self.users.read().await;
        Ok(users.values().find(|u| u.id == *user_id).cloned())
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
