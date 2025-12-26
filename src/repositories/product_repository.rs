use chrono::{DateTime, Utc};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

use crate::{application::product_service::ProductRepository, domain::product::Product};

#[derive(FromRow)]
struct PgProductModel {
    id: Uuid,
    name: String,
    description: String,
    price: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
impl From<PgProductModel> for Product {
    fn from(value: PgProductModel) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            price: value.price as u32,
        }
    }
}

pub struct PgProductRepository {
    pool: PgPool,
}
impl PgProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
impl ProductRepository for PgProductRepository {
    type Error = sqlx::Error;

    async fn create(
        &self,
        name: String,
        description: String,
        price: u32,
    ) -> Result<Product, Self::Error> {
        sqlx::query_as::<_, PgProductModel>(
            "INSERT INTO products (name, description, price) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(name)
        .bind(description)
        .bind(price as i32)
        .fetch_one(&self.pool)
        .await
        .map(|model| model.into())
    }

    async fn read_all(&self) -> Result<Vec<Product>, Self::Error> {
        sqlx::query_as::<_, PgProductModel>("SELECT * FROM products ORDER BY updated_at DESC")
            .fetch_all(&self.pool)
            .await
            .map(|vec| vec.into_iter().map(|model| model.into()).collect())
    }

    async fn read_one(&self, id: Uuid) -> Result<Option<Product>, Self::Error> {
        sqlx::query_as::<_, PgProductModel>("SELECT * FROM products WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map(|opt| opt.map(|model| model.into()))
    }

    async fn update(
        &self,
        id: Uuid,
        name: String,
        description: String,
        price: u32,
    ) -> Result<Option<Product>, Self::Error> {
        sqlx::query_as::<_, PgProductModel>(
            "UPDATE products SET name=$1, description=$2, price=$3, updated_at=now() WHERE id=$4 RETURNING *",
        )
        .bind(name)
        .bind(description)
        .bind(price as i32)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|opt| opt.map(|model| model.into()))
    }

    async fn delete(&self, id: Uuid) -> Result<bool, Self::Error> {
        sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|res| {
                if res.rows_affected() == 0 {
                    false
                } else {
                    true
                }
            })
    }
}
