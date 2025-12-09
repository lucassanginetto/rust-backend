use std::{
    env::{self, VarError},
    error::Error as StdError,
    sync::Mutex,
};

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, web::Data};
use redis::{Client, aio::ConnectionManager};
use sqlx::PgPool;

#[actix_web::get("/")]
async fn hello() -> HttpResponse {
    HttpResponse::Ok().body("Hello there")
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let _ = dotenvy::dotenv();

    let host = "127.0.0.1";
    let port = match env::var("PORT") {
        Err(VarError::NotPresent) => 8080u16,
        r => r?.parse()?,
    };

    let postgres_url = env::var("POSTGRES_URL")?;
    let pg_pool = PgPool::connect(&postgres_url).await?;

    let redis_url = env::var("REDIS_URL")?;
    let redis_client = Client::open(redis_url)?;
    let redis_connman = ConnectionManager::new(redis_client).await?;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(Data::new(pg_pool.clone()))
            .app_data(Data::new(Mutex::new(redis_connman.clone())))
            .service(hello)
    })
    .bind((host, port))?
    .run()
    .await?;

    Ok(())
}
