use crate::entities::player;
use crate::entities::player::AuthResultDto;
use crate::utils::error::Error;
use crate::utils::jwt::Claims;
use actix_web::error::ErrorInternalServerError;
use actix_web::{HttpResponse, Responder, delete, get, patch, post, web};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CreateRequest {
    name: String,
    password: String,
}

/// Register a new player
#[post("/players")]
async fn create(
    db: web::Data<DatabaseConnection>,
    request: web::Json<CreateRequest>,
) -> actix_web::Result<impl Responder> {
    let res = player::create(&db, request.0.name, request.0.password).await;

    match res {
        Ok(entity) => Ok(HttpResponse::Created().json(player::Dto::from(entity))),
        Err(Error::Validation(msg)) => Ok(HttpResponse::BadRequest().body(msg)),
        Err(Error::Duplicate(msg)) => Ok(HttpResponse::Conflict().body(msg)),
        Err(err) => {
            log::error!("POST /players failed: {err:?}");
            Err(ErrorInternalServerError("An unexpected error occurred"))
        }
    }
}

/// Return the authenticated player
#[get("/players/me")]
async fn get_me(
    db: web::Data<DatabaseConnection>,
    claims: Claims,
) -> actix_web::Result<impl Responder> {
    let res = player::get(&db, claims.sub).await;

    match res {
        Ok(entity) => Ok(HttpResponse::Ok().json(player::Dto::from(entity))),
        Err(Error::NotFound()) => Ok(HttpResponse::NotFound().finish()),
        Err(err) => {
            log::error!("GET /players/me failed: {err:?}");
            Err(ErrorInternalServerError("An unexpected error occurred"))
        }
    }
}

/// Return a player by ID
#[get("/players/{id}")]
async fn get_player(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let res = player::get(&db, path.into_inner()).await;

    match res {
        Ok(entity) => Ok(HttpResponse::Ok().json(player::Dto::from(entity))),
        Err(Error::NotFound()) => Ok(HttpResponse::NotFound().finish()),
        Err(err) => {
            log::error!("GET /players/{{id}} failed: {err:?}");
            Err(ErrorInternalServerError("An unexpected error occurred"))
        }
    }
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    name: String,
    password: String,
}

/// Authenticate with name and password, returning tokens
#[post("/players/me/auth")]
async fn auth(
    db: web::Data<DatabaseConnection>,
    request: web::Json<AuthRequest>,
) -> actix_web::Result<impl Responder> {
    let res = player::authenticate(&db, request.0.name, request.0.password).await;

    match res {
        Ok(jwt) => Ok(HttpResponse::Ok().json(AuthResultDto::from(jwt))),
        Err(Error::Authorization()) => Ok(HttpResponse::Unauthorized().finish()),
        Err(err) => {
            log::error!("POST /players/me/auth failed: {err:?}");
            Err(ErrorInternalServerError("An unexpected error occurred"))
        }
    }
}

#[derive(Debug, Deserialize)]
struct RefreshTokensRequest {
    refresh_token: String,
}

/// Exchange a refresh token for a new set of tokens
#[post("/players/me/refresh")]
async fn refresh_tokens(
    db: web::Data<DatabaseConnection>,
    request: web::Json<RefreshTokensRequest>,
) -> actix_web::Result<impl Responder> {
    let res = player::refresh_tokens(&db, request.0.refresh_token).await;

    match res {
        Ok(jwt) => Ok(HttpResponse::Ok().json(AuthResultDto::from(jwt))),
        Err(Error::Authorization()) | Err(Error::NotFound()) => {
            Ok(HttpResponse::Unauthorized().finish())
        }
        Err(err) => {
            log::error!("POST /players/me/auth failed: {err:?}");
            Err(ErrorInternalServerError("An unexpected error occurred"))
        }
    }
}

#[derive(Debug, Deserialize)]
struct ChangeNameRequest {
    name: String,
}

/// Change the authenticated player's name
#[patch("/players/me/name")]
async fn change_name(
    db: web::Data<DatabaseConnection>,
    claims: Claims,
    request: web::Json<ChangeNameRequest>,
) -> actix_web::Result<impl Responder> {
    let res = player::change_name(&db, claims.sub, request.0.name).await;

    match res {
        Ok(entity) => Ok(HttpResponse::Ok().json(player::Dto::from(entity))),
        Err(Error::NotFound()) => Ok(HttpResponse::NotFound().finish()),
        Err(Error::Validation(msg)) => Ok(HttpResponse::BadRequest().body(msg)),
        Err(Error::Duplicate(msg)) => Ok(HttpResponse::Conflict().body(msg)),
        Err(err) => {
            log::error!("PATCH /players/me/name failed: {err:?}");
            Err(ErrorInternalServerError("An unexpected error occurred"))
        }
    }
}

#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

/// Change the authenticated player's password
#[patch("/players/me/password")]
async fn change_password(
    db: web::Data<DatabaseConnection>,
    claims: Claims,
    request: web::Json<ChangePasswordRequest>,
) -> actix_web::Result<impl Responder> {
    let res = player::change_password(
        &db,
        claims.sub,
        request.0.old_password,
        request.0.new_password,
    )
    .await;

    match res {
        Ok(entity) => Ok(HttpResponse::Ok().json(player::Dto::from(entity))),
        Err(Error::NotFound()) => Ok(HttpResponse::NotFound().finish()),
        Err(Error::Validation(msg)) => Ok(HttpResponse::BadRequest().body(msg)),
        Err(Error::Authorization()) => Ok(HttpResponse::Unauthorized().finish()),
        Err(err) => {
            log::error!("PATCH /players/me/password failed: {err:?}");
            Err(ErrorInternalServerError("An unexpected error occurred"))
        }
    }
}

#[derive(Debug, Deserialize)]
struct DeleteRequest {
    password: String,
}

/// Delete the authenticated player
#[delete("/players/me")]
async fn delete(
    db: web::Data<DatabaseConnection>,
    claims: Claims,
    request: web::Json<DeleteRequest>,
) -> actix_web::Result<impl Responder> {
    let res = player::delete(&db, claims.sub, request.0.password).await;

    match res {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(Error::NotFound()) => Ok(HttpResponse::NotFound().finish()),
        Err(Error::Validation(msg)) => Ok(HttpResponse::BadRequest().body(msg)),
        Err(Error::Authorization()) => Ok(HttpResponse::Unauthorized().finish()),
        Err(err) => {
            log::error!("DELETE /players/me failed: {err:?}");
            Err(ErrorInternalServerError("An unexpected error occurred"))
        }
    }
}
