use sea_orm::DbErr;

/// Errors returned by the service's business logic
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum Error {
    /// Input failed validation
    Validation(String),
    /// A uniqueness constraint was violated
    Duplicate(String),
    /// Authentication or authorization failed
    Authorization(),
    /// The requested entity does not exist
    NotFound(),
    /// A unexpected database error
    Db(DbErr),
    /// Any other unexpected error
    Unexpected(String),
}
