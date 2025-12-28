use std::{
    env::{self, VarError},
    error::Error as StdError,
};

use actix_cors::Cors;
use actix_web::{
    App, HttpServer,
    web::{self, Data},
};
use sqlx::PgPool;

use rust_backend::{
    application::product_service::ProductService,
    handlers::product_handlers::{
        add_product, find_product, list_products, put_product, remove_product,
    },
    repositories::product_repository::PgProductRepository,
};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let _ = dotenvy::dotenv();

    env_logger::init();

    let host = "127.0.0.1";
    let port = match env::var("PORT") {
        Err(VarError::NotPresent) => 8080u16,
        result => result?.parse()?,
    };

    let postgres_url = env::var("DATABASE_URL")?;
    let pg_pool = PgPool::connect(&postgres_url).await?;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        type Repo = PgProductRepository;
        let repo = Repo::new(pg_pool.clone());
        let service = ProductService::new(repo);

        App::new().wrap(cors).app_data(Data::new(service)).service(
            web::scope("/api/products")
                .route("", web::get().to(list_products::<Repo>))
                .route("", web::post().to(add_product::<Repo>))
                .route("/{id}", web::get().to(find_product::<Repo>))
                .route("/{id}", web::put().to(put_product::<Repo>))
                .route("/{id}", web::delete().to(remove_product::<Repo>)),
        )
    })
    .bind((host, port))?
    .run()
    .await?;

    Ok(())
}
