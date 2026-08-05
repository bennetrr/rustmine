use actix_web::error::ErrorServiceUnavailable;
use actix_web::{HttpResponse, Responder, get, web};
use sea_orm::DatabaseConnection;

/// Liveness check that pings the database
#[get("/healthz")]
async fn healthz(db: web::Data<DatabaseConnection>) -> actix_web::Result<impl Responder> {
    db.ping().await.map_err(|err| {
        log::error!("GET /healthz failed: Failed to ping database: {err:?}");
        ErrorServiceUnavailable(err)
    })?;

    Ok(HttpResponse::Ok().body("Healthy"))
}
