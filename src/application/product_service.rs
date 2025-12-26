use std::error::Error;

use uuid::Uuid;

use crate::domain::product::Product;

pub trait ProductRepository {
    type Error: Error;

    async fn create(
        &self,
        name: String,
        description: String,
        price: u32,
    ) -> Result<Product, Self::Error>;

    async fn read_all(&self) -> Result<Vec<Product>, Self::Error>;

    async fn read_one(&self, id: Uuid) -> Result<Option<Product>, Self::Error>;

    async fn update(
        &self,
        id: Uuid,
        name: String,
        description: String,
        price: u32,
    ) -> Result<Option<Product>, Self::Error>;

    async fn delete(&self, id: Uuid) -> Result<bool, Self::Error>;
}

pub enum ProductServiceError<E> {
    NotFound,
    Repository(E),
}

pub struct ProductService<R: ProductRepository> {
    repo: R,
}
impl<R: ProductRepository> ProductService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn add(
        &self,
        name: String,
        description: String,
        price: u32,
    ) -> Result<Product, R::Error> {
        self.repo.create(name, description, price).await
    }

    pub async fn list(&self) -> Result<Vec<Product>, R::Error> {
        self.repo.read_all().await
    }

    pub async fn find(&self, id: Uuid) -> Result<Product, ProductServiceError<R::Error>> {
        self.repo
            .read_one(id)
            .await
            .map_err(ProductServiceError::Repository)
            .and_then(|opt| {
                if let Some(product) = opt {
                    Ok(product)
                } else {
                    Err(ProductServiceError::NotFound)
                }
            })
    }

    pub async fn modify(
        &self,
        id: Uuid,
        name: String,
        description: String,
        price: u32,
    ) -> Result<Product, ProductServiceError<R::Error>> {
        self.repo
            .update(id, name, description, price)
            .await
            .map_err(ProductServiceError::Repository)
            .and_then(|opt| {
                if let Some(product) = opt {
                    Ok(product)
                } else {
                    Err(ProductServiceError::NotFound)
                }
            })
    }

    pub async fn remove(&self, id: Uuid) -> Result<(), ProductServiceError<R::Error>> {
        self.repo
            .delete(id)
            .await
            .map_err(ProductServiceError::Repository)
            .and_then(|found| {
                if found {
                    Ok(())
                } else {
                    Err(ProductServiceError::NotFound)
                }
            })
    }
}
