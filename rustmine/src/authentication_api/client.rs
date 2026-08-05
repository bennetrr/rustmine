#![allow(unused)]
use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::error::Error;
use super::models::{AuthResult, Player};

/// Blocking HTTP client for the RustMine authentication service
pub struct AuthenticationApi {
    base_url: String,
    client: Client,
}

#[derive(Serialize)]
struct CreatePlayerRequest<'a> {
    name: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct AuthenticateRequest<'a> {
    name: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct RefreshRequest<'a> {
    refresh_token: &'a str,
}

#[derive(Serialize)]
struct ChangeNameRequest<'a> {
    name: &'a str,
}

#[derive(Serialize)]
struct ChangePasswordRequest<'a> {
    old_password: &'a str,
    new_password: &'a str,
}

#[derive(Serialize)]
struct DeleteRequest<'a> {
    password: &'a str,
}

impl AuthenticationApi {
    /// Create a client targeting the given base URL (e.g. `http://localhost:65432`)
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    /// Create a client with a pre-configured [`reqwest::blocking::Client`]
    pub fn with_client(base_url: impl Into<String>, client: Client) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client,
        }
    }

    /// Register a new player
    pub fn create_player(&self, name: &str, password: &str) -> Result<Player, Error> {
        let resp = self
            .client
            .post(self.url("/players"))
            .json(&CreatePlayerRequest { name, password })
            .send()?;
        json(resp)
    }

    /// Authenticate with name and password
    pub fn authenticate(&self, name: &str, password: &str) -> Result<AuthResult, Error> {
        let resp = self
            .client
            .post(self.url("/players/me/auth"))
            .json(&AuthenticateRequest { name, password })
            .send()?;
        json(resp)
    }

    /// Exchange a refresh token for a new set of tokens
    pub fn refresh(&self, refresh_token: &str) -> Result<AuthResult, Error> {
        let resp = self
            .client
            .post(self.url("/players/me/refresh"))
            .json(&RefreshRequest { refresh_token })
            .send()?;
        json(resp)
    }

    /// Get the authenticated player
    pub fn get_me(&self, access_token: &str) -> Result<Player, Error> {
        let resp = self
            .client
            .get(self.url("/players/me"))
            .bearer_auth(access_token)
            .send()?;
        json(resp)
    }

    /// Get a player by ID
    pub fn get_player(&self, access_token: &str, id: &str) -> Result<Player, Error> {
        let resp = self
            .client
            .get(self.url(&format!("/players/{id}")))
            .bearer_auth(access_token)
            .send()?;
        json(resp)
    }

    /// Change the authenticated player's name
    pub fn change_name(&self, access_token: &str, name: &str) -> Result<Player, Error> {
        let resp = self
            .client
            .patch(self.url("/players/me/name"))
            .bearer_auth(access_token)
            .json(&ChangeNameRequest { name })
            .send()?;
        json(resp)
    }

    /// Change the authenticated player's password
    pub fn change_password(
        &self,
        access_token: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<Player, Error> {
        let resp = self
            .client
            .patch(self.url("/players/me/password"))
            .bearer_auth(access_token)
            .json(&ChangePasswordRequest {
                old_password,
                new_password,
            })
            .send()?;
        json(resp)
    }

    /// Delete the authenticated player
    pub fn delete(&self, access_token: &str, password: &str) -> Result<(), Error> {
        let resp = self
            .client
            .delete(self.url("/players/me"))
            .bearer_auth(access_token)
            .json(&DeleteRequest { password })
            .send()?;
        empty(resp)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }
}

/// Map a successful response body to `T`, or the status code to an [`Error`]
fn json<T: DeserializeOwned>(resp: Response) -> Result<T, Error> {
    check(resp)?.json::<T>().map_err(Error::from)
}

/// Like [`json`] but for endpoints that return no body (e.g. 204 No Content)
fn empty(resp: Response) -> Result<(), Error> {
    check(resp).map(|_| ())
}

/// Translate non-success status codes into the matching [`Error`] variant
fn check(resp: Response) -> Result<Response, Error> {
    match resp.status() {
        s if s.is_success() => Ok(resp),
        StatusCode::BAD_REQUEST => Err(Error::Validation(resp.text().unwrap_or_default())),
        StatusCode::UNAUTHORIZED => Err(Error::Unauthorized),
        StatusCode::NOT_FOUND => Err(Error::NotFound),
        StatusCode::CONFLICT => Err(Error::Conflict(resp.text().unwrap_or_default())),
        other => Err(Error::Unexpected(other)),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json as json_macro;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[allow(dead_code)]
    #[derive(serde::Deserialize, Serialize, Clone)]
    pub struct MockPlayer {
        pub id: String,
        pub name: String,
    }

    // Create player test
    #[test]
    fn test_create_player_success() {
        // Build a persistent runtime instance for the duration of this test execution scope
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let mock_server = runtime.block_on(MockServer::start());

        let expected_player = json_macro!({
            "id": "player-123",
            "name": "Steve"
        });

        runtime.block_on(async {
            Mock::given(method("POST"))
                .and(path("/players"))
                .and(body_json(
                    json_macro!({ "name": "Steve", "password": "secure123" }),
                ))
                .respond_with(ResponseTemplate::new(201).set_body_json(expected_player))
                .mount(&mock_server)
                .await;
        });

        let api = AuthenticationApi::new(mock_server.uri());
        let result = api.create_player("Steve", "secure123");

        assert!(
            result.is_ok(),
            "Expected production endpoint to return player payload, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_get_me_with_bearer_token() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let mock_server = runtime.block_on(MockServer::start());
        let token = "secret_jwt_token";

        runtime.block_on(async {
            Mock::given(method("GET"))
                .and(path("/players/me"))
                .and(header("Authorization", &format!("Bearer {token}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(json_macro!({
                    "id": "1", "name": "Alex"
                })))
                .mount(&mock_server)
                .await;
        });

        let api = AuthenticationApi::new(mock_server.uri());
        let result = api.get_me(token);
        assert!(
            result.is_ok(),
            "Expected authentication token verification to pass, got: {:?}",
            result.err()
        );
    }

    // authenticate test
    #[test]
    fn test_authenticate_success() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let mock_server = runtime.block_on(MockServer::start());

        let expected_auth = json_macro!({
            "access_token": "valid_access",
            "refresh_token": "valid_refresh",
             "id_token": "..."
        });

        runtime.block_on(async {
            Mock::given(method("POST"))
                .and(path("/players/me/auth"))
                .respond_with(ResponseTemplate::new(200).set_body_json(expected_auth))
                .mount(&mock_server)
                .await;
        });

        let api = AuthenticationApi::new(mock_server.uri());
        let result = api.authenticate("Steve", "secure123");

        assert!(
            result.is_ok(),
            "Expected client authentication response mapping to match, got: {:?}",
            result.err()
        );
    }

    // delete account test
    #[test]
    fn test_delete_account_empty_body() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let mock_server = runtime.block_on(MockServer::start());

        runtime.block_on(async {
            Mock::given(method("DELETE"))
                .and(path("/players/me"))
                .respond_with(ResponseTemplate::new(204)) // HTTP 204 No Content
                .mount(&mock_server)
                .await;
        });

        let api = AuthenticationApi::new(mock_server.uri());
        let result = api.delete("token", "password123");

        assert!(
            result.is_ok(),
            "Expected no-content handling mapping to match empty body response parser, got: {:?}",
            result.err()
        );
    }

    // error handling when unauthorized
    #[test]
    fn test_error_handling_unauthorized() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let mock_server = runtime.block_on(MockServer::start());

        runtime.block_on(async {
            Mock::given(method("GET"))
                .and(path("/players/me"))
                .respond_with(ResponseTemplate::new(401)) // Emulate unauthorized response error
                .mount(&mock_server)
                .await;
        });

        let api = AuthenticationApi::new(mock_server.uri());
        let result = api.get_me("invalid_token");

        assert!(
            result.is_err(),
            "Expected error return variants, found generic success instance instead"
        );
        if let Err(Error::Unauthorized) = result {
        } else {
            panic!("Expected Error::Unauthorized variant mapping from status codes");
        }
    }

    #[test]
    fn test_error_handling_validation_bad_request() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let mock_server = runtime.block_on(MockServer::start());
        let error_msg = "Username already taken";

        runtime.block_on(async {
            Mock::given(method("POST"))
                .and(path("/players"))
                .respond_with(ResponseTemplate::new(400).set_body_string(error_msg))
                .mount(&mock_server)
                .await;
        });

        let api = AuthenticationApi::new(mock_server.uri());
        let result = api.create_player("existing_user", "password");

        assert!(result.is_err());
        if let Err(Error::Validation(msg)) = result {
            assert_eq!(msg, error_msg);
        } else {
            panic!("Expected Error::Validation variant container mapping bad request details");
        }
    }
}
