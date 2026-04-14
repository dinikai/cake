use std::fmt::Display;

use crate::cmd::Request;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Wrapper for `Request`. Carries an auth token.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AuthRequestEnvelope {
    pub auth_token: AuthToken,
    pub request: Request,
}

impl AuthRequestEnvelope {
    /// Constructs new envelope from request and token.
    pub fn from(request: &Request, token: &AuthToken) -> Self {
        Self {
            auth_token: token.clone(),
            request: request.clone(),
        }
    }
}

/// Holds an authentication token info.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AuthToken {
    pub uuid: Uuid,
}

impl AuthToken {
    /// Creates new unique token.
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }

    pub fn from(uuid: &Uuid) -> Self {
        Self { uuid: uuid.clone() }
    }
}

impl Display for AuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uuid)
    }
}

impl PartialEq for AuthToken {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}
