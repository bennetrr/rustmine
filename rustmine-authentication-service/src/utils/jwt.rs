use crate::utils::error::Error;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::http::header::AUTHORIZATION;
use actix_web::middleware::Next;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env::var;
use std::future::{Ready, ready};
use std::sync::OnceLock;
use time::{Duration, OffsetDateTime};

const ISSUER: &str = "rustmine-authentication-service";
const TOKEN_TTL: Duration = Duration::minutes(10);
const REFRESH_TOKEN_TTL: Duration = Duration::weeks(2);

#[derive(Clone, Copy, Debug)]
pub(crate) enum TokenKind {
    /// Access token used to authenticate API calls
    Access,
    /// ID token used to authenticate against multiplayer hosts
    Id,
    /// Refresh token to retrieve a new set of tokens from the API
    Refresh,
}

impl TokenKind {
    /// The time-to-live (TTL) of the token type
    fn ttl(self) -> Duration {
        match self {
            TokenKind::Refresh => REFRESH_TOKEN_TTL,
            TokenKind::Access | TokenKind::Id => TOKEN_TTL,
        }
    }

    /// Value of the JWT `typ` header, used to bind a token to its purpose
    fn typ(self) -> &'static str {
        match self {
            TokenKind::Access => "at+jwt",
            TokenKind::Id => "id_token+jwt",
            TokenKind::Refresh => "refresh+jwt",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    /// Player ID
    pub sub: String,
    /// Player name
    pub name: String,
    /// Issuer
    pub iss: String,
    /// Issued at timestamp
    pub iat: u64,
    /// Expiry timestamp
    pub exp: u64,
}

impl FromRequest for Claims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        ready(req.extensions().get::<Claims>().cloned().ok_or_else(|| {
            ErrorInternalServerError("Claims missing; route is not wrapped in `authenticated`")
        }))
    }
}

/// Get the private key for signing JWTs
fn private_key() -> Result<&'static EncodingKey, Error> {
    static KEY: OnceLock<Result<EncodingKey, String>> = OnceLock::new();
    KEY.get_or_init(|| {
        let pem = var("JWT_PRIVATE_KEY").map_err(|_| "JWT_PRIVATE_KEY not set".to_string())?;
        EncodingKey::from_ed_pem(pem.as_bytes()).map_err(|e| e.to_string())
    })
    .as_ref()
    .map_err(|e| Error::Unexpected(e.clone()))
}

/// Get the public key for signing JWTs
fn public_key() -> Result<&'static DecodingKey, Error> {
    static KEY: OnceLock<Result<DecodingKey, String>> = OnceLock::new();
    KEY.get_or_init(|| {
        let pem = var("JWT_PUBLIC_KEY").map_err(|_| "JWT_PUBLIC_KEY not set".to_string())?;
        DecodingKey::from_ed_pem(pem.as_bytes()).map_err(|e| e.to_string())
    })
    .as_ref()
    .map_err(|e| Error::Unexpected(e.clone()))
}

/// Create a signed JWT with the given values
///
/// # Arguments
///
/// - `id`: The player ID
/// - `name`: The player name
/// - `kind`: The token kind
pub(crate) fn create_jwt(id: String, name: String, kind: TokenKind) -> Result<String, Error> {
    let now = OffsetDateTime::now_utc();
    let exp = now + kind.ttl();

    let claims = Claims {
        sub: id,
        name,
        iss: ISSUER.to_string(),
        iat: now.unix_timestamp() as u64,
        exp: exp.unix_timestamp() as u64,
    };

    let mut header = Header::new(Algorithm::EdDSA);
    header.typ = Some(kind.typ().to_string());

    encode(&header, &claims, private_key()?).map_err(|e| Error::Unexpected(e.to_string()))
}

/// Validate a JWT and return its claims
///
/// # Arguments
///
/// - `token`: The JWT
/// - `kind`: The expected token kind
pub(crate) fn verify_jwt(token: &str, kind: TokenKind) -> Result<Claims, Error> {
    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.set_issuer(&[ISSUER]);
    let data =
        decode::<Claims>(token, public_key()?, &validation).map_err(|_| Error::Authorization())?;

    if data.header.typ.as_deref() != Some(kind.typ()) {
        return Err(Error::Authorization());
    }

    Ok(data.claims)
}

/// Actix Web middleware that checks for authentication and adds the JWT claims to the request data
pub(crate) async fn authenticated(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| ErrorUnauthorized("missing or malformed Authorization header"))?;

    let claims =
        verify_jwt(token, TokenKind::Access).map_err(|_| ErrorUnauthorized("invalid token"))?;
    req.extensions_mut().insert(claims);

    next.call(req).await
}
