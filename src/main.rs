mod error;
mod routes;
mod structs;

use actix_web::{web::Data, App, HttpServer};
use actix_web::middleware::Logger;
use anyhow::Result;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::{sync::Mutex, time::Duration};
use structs::AppState;


#[actix_web::main]
async fn main() -> Result<()> {
    // Leave debug stuff for now
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    dotenv().ok();

    // Create a connection pool to our database
    let db_url = std::env::var("DB_URL")?;
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .max_lifetime(Duration::new(6, 0)) // max db connection lifetime of 6 seconds for now
        .connect(&db_url)
        .await?;

    // A shared app state among requests for tracking active connections,
    // database connection pool, and the max payload size.
    let app_state = Data::new(AppState {
        active_cnx: Mutex::new(0),
        max_payload_size: 262_144,
        db_pool,
    });

    // Build, Setup, & Start The Api (HTTP SERVER)
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_state.clone())
            .configure(routes::config::configure_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?)
}

