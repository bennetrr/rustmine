use crate::utils::error::Error;
use crate::utils::jwt::{TokenKind, create_jwt, verify_jwt};
use crate::utils::password::{hash_password, verify_password};
use sea_orm::IntoActiveValue;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "players")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    #[sea_orm(unique)]
    pub name: String,
    pub password_hash: String,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dto {
    pub id: String,
    pub name: String,
}

impl From<Model> for Dto {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
        }
    }
}

/// Validate a player name
fn validate_name(name: &str) -> Result<(), Error> {
    if name.trim().is_empty() {
        return Err(Error::Validation("name must not be empty".to_string()));
    }

    if name.len() > 30 {
        return Err(Error::Validation(
            "name must not be longer than 30 characters".to_string(),
        ));
    }

    Ok(())
}

/// Validate a password
fn validate_password(password: &str) -> Result<(), Error> {
    if password.trim().is_empty() {
        return Err(Error::Validation("password must not be empty".to_string()));
    }

    if password.len() < 12 {
        return Err(Error::Validation(
            "password must be 12 chars or longer".to_string(),
        ));
    }

    Ok(())
}

/// Create a new player
pub(crate) async fn create(
    db: &DatabaseConnection,
    name: String,
    password: String,
) -> Result<Model, Error> {
    validate_name(&name)?;
    validate_password(&password)?;

    let player = ActiveModel {
        id: Uuid::new_v4().to_string().into_active_value(),
        name: name.into_active_value(),
        password_hash: hash_password(&password)?.into_active_value(),
    };

    player.insert(db).await.map_err(|e| match e.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => {
            Error::Duplicate("A user with the same name is already registered".to_string())
        }
        _ => Error::Db(e),
    })
}

/// Get a player by ID
pub(crate) async fn get(db: &DatabaseConnection, id: String) -> Result<Model, Error> {
    Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(Error::Db)?
        .ok_or_else(Error::NotFound)
}

pub(crate) struct AuthResult {
    access_token: String,
    id_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AuthResultDto {
    access_token: String,
    id_token: String,
    refresh_token: String,
}

impl From<AuthResult> for AuthResultDto {
    fn from(value: AuthResult) -> Self {
        AuthResultDto {
            access_token: value.access_token,
            id_token: value.id_token,
            refresh_token: value.refresh_token,
        }
    }
}

/// Issue a fresh set of access, ID and refresh tokens for a player
fn issue_tokens(id: &str, name: &str) -> Result<AuthResult, Error> {
    Ok(AuthResult {
        access_token: create_jwt(id.to_string(), name.to_string(), TokenKind::Access)?,
        id_token: create_jwt(id.to_string(), name.to_string(), TokenKind::Id)?,
        refresh_token: create_jwt(id.to_string(), name.to_string(), TokenKind::Refresh)?,
    })
}

/// Authenticate the player
pub(crate) async fn authenticate(
    db: &DatabaseConnection,
    name: String,
    password: String,
) -> Result<AuthResult, Error> {
    let player = Entity::find()
        .filter(Column::Name.eq(name))
        .one(db)
        .await
        .map_err(Error::Db)?
        .ok_or_else(Error::Authorization)?;

    if !verify_password(&password, &player.password_hash) {
        return Err(Error::Authorization());
    }

    issue_tokens(&player.id, &player.name)
}

/// Issue a new set of tokens from a valid refresh token
pub(crate) async fn refresh_tokens(
    db: &DatabaseConnection,
    old_refresh_token: String,
) -> Result<AuthResult, Error> {
    let claims = verify_jwt(old_refresh_token.as_str(), TokenKind::Refresh)?;
    let player = get(db, claims.sub).await?;

    issue_tokens(&player.id, &player.name)
}

/// Change a player's name
pub(crate) async fn change_name(
    db: &DatabaseConnection,
    id: String,
    name: String,
) -> Result<Model, Error> {
    validate_name(&name)?;

    let mut player: ActiveModel = get(db, id).await?.into();
    player.name = name.into_active_value();

    player.update(db).await.map_err(|e| match e.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => {
            Error::Duplicate("A user with the same name is already registered".to_string())
        }
        _ => Error::Db(e),
    })
}

/// Change a player's password after verifying the old one
pub(crate) async fn change_password(
    db: &DatabaseConnection,
    id: String,
    old_password: String,
    new_password: String,
) -> Result<Model, Error> {
    validate_password(&new_password)?;

    let mut player: ActiveModel = get(db, id).await?.into();

    if !verify_password(&old_password, &player.password_hash.unwrap()) {
        return Err(Error::Authorization());
    }

    player.password_hash = hash_password(&new_password)?.into_active_value();

    player.update(db).await.map_err(Error::Db)
}

/// Delete a player after verifying their password
pub(crate) async fn delete(
    db: &DatabaseConnection,
    id: String,
    password: String,
) -> Result<(), Error> {
    let player = get(db, id).await?;

    if !verify_password(&password, &player.password_hash) {
        return Err(Error::Authorization());
    }

    let player: ActiveModel = player.into();
    player.delete(db).await.map_err(Error::Db)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Password test
    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("valid_name").is_ok());
        assert!(validate_name("a").is_ok());
        assert!(validate_name("a".repeat(30).as_str()).is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        let err = validate_name("").unwrap_err();
        assert!(matches!(err, Error::Validation(msg) if msg.contains("must not be empty")));

        let err = validate_name("   ").unwrap_err();
        assert!(matches!(err, Error::Validation(msg) if msg.contains("must not be empty")));
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(31);
        let err = validate_name(&long_name).unwrap_err();
        assert!(matches!(err, Error::Validation(msg) if msg.contains("longer than 30")));
    }

    #[test]
    fn test_validate_password_valid() {
        assert!(validate_password("password123456").is_ok());
        assert!(validate_password("a".repeat(12).as_str()).is_ok());
        assert!(validate_password("a".repeat(100).as_str()).is_ok());
    }

    #[test]
    fn test_validate_password_empty() {
        let err = validate_password("").unwrap_err();
        assert!(matches!(err, Error::Validation(msg) if msg.contains("must not be empty")));

        let err = validate_password("   ").unwrap_err();
        assert!(matches!(err, Error::Validation(msg) if msg.contains("must not be empty")));
    }

    #[test]
    fn test_validate_password_too_short() {
        let err = validate_password("short").unwrap_err();
        assert!(matches!(err, Error::Validation(msg) if msg.contains("12 chars")));
    }

    #[test]
    fn test_dto_from_model() {
        let model = Model {
            id: "test-id".to_string(),
            name: "test-player".to_string(),
            password_hash: "hash".to_string(),
        };

        let dto: Dto = model.into();
        assert_eq!(dto.id, "test-id");
        assert_eq!(dto.name, "test-player");
    }

    #[test]
    fn test_auth_result_into_dto() {
        let auth_result = AuthResult {
            access_token: "access".to_string(),
            id_token: "id".to_string(),
            refresh_token: "refresh".to_string(),
        };

        let dto: AuthResultDto = auth_result.into();
        assert_eq!(dto.access_token, "access");
        assert_eq!(dto.id_token, "id");
        assert_eq!(dto.refresh_token, "refresh");
    }

    #[test]
    fn test_change_name_validation() {
        let err = validate_name("").unwrap_err();
        assert!(matches!(err, Error::Validation(_)));

        let err = validate_name("a".repeat(31).as_str()).unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[test]
    fn test_change_password_validation() {
        let err = validate_password("short").unwrap_err();
        assert!(matches!(err, Error::Validation(_)));

        let err = validate_password("hause").unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[test]
    fn test_model_derives() {
        let player1 = Model {
            id: "1".to_string(),
            name: "player1".to_string(),
            password_hash: "hash1".to_string(),
        };
        let player2 = Model {
            id: "1".to_string(),
            name: "player1".to_string(),
            password_hash: "hash1".to_string(),
        };

        assert_eq!(player1, player2);
        assert!(player1.id == player2.id);
        assert!(player1.name == player2.name);
    }
}
