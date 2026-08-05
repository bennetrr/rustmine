use serde::{Deserialize, Serialize};

/// A player as returned by the authentication service
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
}

/// The set of tokens returned after authenticating or refreshing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthResult {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: String,
}
