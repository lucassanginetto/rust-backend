use std::sync::Mutex;

use actix_web::{
    HttpResponse, Responder,
    http::header::LOCATION,
    web::{Data, Json, Path},
};
use redis::aio::ConnectionManager;
use sqlx::{PgPool, Row, postgres::PgRow};
use uuid::Uuid;

use crate::{
    cache::{self, DEFAULT_EXPIRATION as CACHE_EXPIRATION},
    product::{CreateProductDTO, Product, UpdateProductDTO},
};

#[actix_web::get("/")]
async fn hello() -> HttpResponse {
    HttpResponse::Ok().body("Hello there")
}

#[actix_web::get("/api/products")]
async fn get_products(pool: Data<PgPool>, redis: Data<Mutex<ConnectionManager>>) -> impl Responder {
    const KEY: &str = "products";

    if let Ok(Some(cached)) = cache::get::<Vec<Product>>(KEY, &redis).await {
        return HttpResponse::Ok().json(cached);
    }

    let rows = sqlx::query_as::<_, Product>("SELECT * FROM products ORDER BY updated_at DESC")
        .fetch_all(pool.get_ref())
        .await;

    match rows {
        Ok(products) => {
            let _ = cache::set(KEY, &products, CACHE_EXPIRATION, &redis).await;
            HttpResponse::Ok().json(products)
        }
        Err(error) => {
            log::error!("error while retrieving products: {}", error);
            HttpResponse::InternalServerError()
                .body("The server was unable to retrieve product due to an internal error")
        }
    }
}

#[actix_web::get("/api/products/{id}")]
async fn get_product(
    uuid: Path<Uuid>,
    pool: Data<PgPool>,
    redis: Data<Mutex<ConnectionManager>>,
) -> impl Responder {
    let id = uuid.into_inner();
    let key = format!("products:{}", id);

    if let Ok(Some(cached)) = cache::get::<Product>(&key, &redis).await {
        return HttpResponse::Ok().json(cached);
    }

    let row = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
        .bind(id)
        .fetch_one(pool.get_ref())
        .await;

    match row {
        Ok(product) => {
            let _ = cache::set(&key, &product, CACHE_EXPIRATION, &redis).await;
            HttpResponse::Ok().json(product)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Product was not found"),
        Err(error) => {
            log::error!("error while retrieving product: {}", error);
            HttpResponse::InternalServerError()
                .body("The server was unable to retrieve product due to an internal error")
        }
    }
}

#[actix_web::post("/api/products")]
async fn post_product(
    payload: Json<CreateProductDTO>,
    pool: Data<PgPool>,
    redis: Data<Mutex<ConnectionManager>>,
) -> impl Responder {
    if payload.price < 0 {
        return HttpResponse::BadRequest().body("Price can't be negative.");
    }

    let response = sqlx::query(
        "INSERT INTO products (name, description, price) VALUES ($1,$2,$3) RETURNING *",
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.price)
    .map(|row: PgRow| Product {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        price: row.get("price"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
    .fetch_one(pool.get_ref())
    .await;

    match response {
        Ok(product) => {
            let _ = cache::del("products", &redis).await;
            HttpResponse::Created()
                .insert_header((LOCATION, format!("/api/products/{}", product.id)))
                .json(product)
        }
        Err(error) => {
            log::error!("error while creating product: {}", error);
            HttpResponse::InternalServerError()
                .body("The server was unable to create product due to an internal error")
        }
    }
}

#[actix_web::put("/api/products/{id}")]
async fn put_product(
    uuid: Path<Uuid>,
    payload: Json<CreateProductDTO>,
    pool: Data<PgPool>,
    redis: Data<Mutex<ConnectionManager>>,
) -> impl Responder {
    let id = uuid.into_inner();

    if payload.price < 0 {
        return HttpResponse::BadRequest().body("Price can't be negative");
    }

    let response = sqlx::query_as::<_, Product>("UPDATE products SET name = $1, description = $2, price = $3, updated_at = now() WHERE id = $4 RETURNING *").bind(&payload.name).bind(&payload.description).bind(payload.price).bind(id).fetch_one(pool.get_ref()).await;

    match response {
        Ok(product) => {
            let _ = cache::del("products", &redis).await;
            let _ = cache::del(&format!("products:{}", product.id), &redis).await;
            HttpResponse::Ok().json(product)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Product was not found"),
        Err(error) => {
            log::error!("error while replacing product: {}", error);
            HttpResponse::InternalServerError()
                .body("The server was unable to replace product due to an internal error")
        }
    }
}

#[actix_web::patch("/api/products/{id}")]
async fn patch_product(
    uuid: Path<Uuid>,
    payload: Json<UpdateProductDTO>,
    pool: Data<PgPool>,
    redis: Data<Mutex<ConnectionManager>>,
) -> impl Responder {
    let id = uuid.into_inner();
    let dto = payload.into_inner();

    if let Some(price) = dto.price
        && price < 0
    {
        return HttpResponse::BadRequest().body("Price can't be negative");
    }

    let existing = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
        .bind(id)
        .fetch_one(pool.get_ref())
        .await;

    let mut product = match existing {
        Ok(product) => product,
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound().body("Product was not found");
        }
        Err(error) => {
            log::error!("error while updating product: {}", error);
            return HttpResponse::InternalServerError()
                .body("The server was unable to update product due to an internal error");
        }
    };

    if let Some(name) = dto.name {
        product.name = name;
    }
    if let Some(description) = dto.description {
        product.description = description;
    }
    if let Some(price) = dto.price {
        product.price = price;
    }

    let response = sqlx::query_as::<_, Product>(
        "UPDATE products
        SET name = $1, description = $2, price = $3, updated_at = now()
        WHERE id = $4
        RETURNING *",
    )
    .bind(&product.name)
    .bind(&product.description)
    .bind(product.price)
    .bind(id)
    .fetch_one(pool.get_ref())
    .await;

    match response {
        Ok(updated) => {
            let _ = cache::del("products", &redis).await;
            let _ = cache::del(&format!("products:{}", updated.id), &redis).await;
            HttpResponse::Ok().json(updated)
        }
        Err(error) => {
            log::error!("error while updating product: {}", error);
            HttpResponse::InternalServerError()
                .body("The server was unable to update product due to an internal error")
        }
    }
}
