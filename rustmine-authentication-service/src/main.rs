use crate::utils::jwt::authenticated;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer, web};
use sea_orm::Database;
use std::env::var;

pub mod endpoints;
pub(crate) mod entities;
pub(crate) mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    // Set up database connection
    let db_uri = format!(
        "mysql://{}:{}@{}/{}",
        var("MARIADB_USER").expect("Please set the MARIADB_USER environment variable"),
        var("MARIADB_PASSWORD").expect("Please set the MARIADB_PASSWORD environment variable"),
        var("MARIADB_HOST").expect("Please set the MARIADB_HOST environment variable"),
        var("MARIADB_DATABASE").expect("Please set the MARIADB_DATABASE environment variable")
    );

    let db = Database::connect(db_uri)
        .await
        .expect("Failed to connect to the database");

    // Migrate database schemas
    db.get_schema_registry("rustmine-authentication-service::entities::*")
        .sync(&db)
        .await
        .expect("Failed to migrate the database");

    let db_state = db.clone();

    // Start the HTTP server
    let default_port: u16 = 65432;
    let port = var("PORT")
        .unwrap_or(default_port.to_string())
        .parse::<u16>()
        .unwrap_or(default_port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_state.clone()))
            .service(endpoints::healthz::healthz)
            .service(endpoints::player::create)
            .service(endpoints::player::auth)
            .service(endpoints::player::refresh_tokens)
            .service(
                web::scope("")
                    .wrap(from_fn(authenticated))
                    .service(endpoints::player::change_name)
                    .service(endpoints::player::change_password)
                    .service(endpoints::player::get_me)
                    .service(endpoints::player::get_player)
                    .service(endpoints::player::delete),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    // Cleanup resources
    db.close()
        .await
        .expect("Failed to close database connection");

    Ok(())
}
