use sqlx::PgPool;
use uuid::Uuid;

use rust_backend::{
    application::product_service::ProductRepository,
    repositories::product_repository::PgProductRepository,
};

#[sqlx::test(migrations = "./migrations")]
async fn create_product_works(pool: PgPool) {
    let repo = PgProductRepository::new(pool);

    let product = repo
        .create("Book".into(), "A nice book".into(), 100)
        .await
        .unwrap();

    assert_eq!(product.name, "Book");
    assert_eq!(product.price, 100);
}

#[sqlx::test(migrations = "./migrations")]
async fn read_all_returns_products(pool: PgPool) {
    let repo = PgProductRepository::new(pool);

    repo.create("Item A".into(), "Desc".into(), 10)
        .await
        .unwrap();
    repo.create("Item B".into(), "Desc".into(), 20)
        .await
        .unwrap();

    let products = repo.read_all().await.unwrap();

    assert_eq!(products.len(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn read_one_returns_none_if_missing(pool: PgPool) {
    let repo = PgProductRepository::new(pool);

    let result = repo.read_one(Uuid::new_v4()).await.unwrap();

    assert!(result.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn update_product_works(pool: PgPool) {
    let repo = PgProductRepository::new(pool);

    let product = repo
        .create("Old".into(), "Old desc".into(), 10)
        .await
        .unwrap();

    let updated = repo
        .update(product.id, "New".into(), "New desc".into(), 20)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "New");
    assert_eq!(updated.price, 20);
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_product_works(pool: PgPool) {
    let repo = PgProductRepository::new(pool);

    let product = repo.create("Temp".into(), "Temp".into(), 1).await.unwrap();

    let deleted = repo.delete(product.id).await.unwrap();
    assert!(deleted);

    let found = repo.read_one(product.id).await.unwrap();
    assert!(found.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn service_maps_not_found_correctly(pool: PgPool) {
    use rust_backend::{
        application::product_service::{ProductService, ProductServiceError},
        repositories::product_repository::PgProductRepository,
    };

    let repo = PgProductRepository::new(pool);
    let service = ProductService::new(repo);

    let result = service.find(Uuid::new_v4()).await;

    matches!(result, Err(ProductServiceError::NotFound));
}
