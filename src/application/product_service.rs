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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[derive(Default)]
    struct MockProductRepository {
        products: std::sync::Mutex<Vec<Product>>,
        fail: bool,
    }

    #[derive(Debug)]
    struct MockError;
    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Mock repository error")
        }
    }
    impl std::error::Error for MockError {}

    impl ProductRepository for MockProductRepository {
        type Error = MockError;

        async fn create(
            &self,
            name: String,
            description: String,
            price: u32,
        ) -> Result<Product, Self::Error> {
            if self.fail {
                return Err(MockError);
            }

            let product = Product {
                id: Uuid::new_v4(),
                name,
                description,
                price,
            };

            self.products.lock().unwrap().push(product.clone());
            Ok(product)
        }

        async fn read_all(&self) -> Result<Vec<Product>, Self::Error> {
            if self.fail {
                return Err(MockError);
            }

            Ok(self.products.lock().unwrap().clone())
        }

        async fn read_one(&self, id: Uuid) -> Result<Option<Product>, Self::Error> {
            if self.fail {
                return Err(MockError);
            }

            Ok(self
                .products
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id == id)
                .cloned())
        }

        async fn update(
            &self,
            id: Uuid,
            name: String,
            description: String,
            price: u32,
        ) -> Result<Option<Product>, Self::Error> {
            if self.fail {
                return Err(MockError);
            }

            let mut products = self.products.lock().unwrap();
            if let Some(p) = products.iter_mut().find(|p| p.id == id) {
                p.name = name;
                p.description = description;
                p.price = price;
                return Ok(Some(p.clone()));
            }

            Ok(None)
        }

        async fn delete(&self, id: Uuid) -> Result<bool, Self::Error> {
            if self.fail {
                return Err(MockError);
            }

            let mut products = self.products.lock().unwrap();
            let len_before = products.len();
            products.retain(|p| p.id != id);

            Ok(products.len() != len_before)
        }
    }

    #[tokio::test]
    async fn add_product_success() {
        let repo = MockProductRepository::default();
        let service = ProductService::new(repo);

        let product = service
            .add("Book".into(), "A nice book".into(), 1000)
            .await
            .unwrap();

        assert_eq!(product.name, "Book");
        assert_eq!(product.price, 1000);
    }

    #[tokio::test]
    async fn list_products_returns_all() {
        let repo = MockProductRepository::default();
        let service = ProductService::new(repo);

        service
            .add("Item 1".into(), "Desc".into(), 10)
            .await
            .unwrap();
        service
            .add("Item 2".into(), "Desc".into(), 20)
            .await
            .unwrap();

        let products = service.list().await.unwrap();
        assert_eq!(products.len(), 2);
    }

    #[tokio::test]
    async fn find_product_not_found() {
        let repo = MockProductRepository::default();
        let service = ProductService::new(repo);

        let result = service.find(Uuid::new_v4()).await;

        assert!(matches!(result, Err(ProductServiceError::NotFound)));
    }

    #[tokio::test]
    async fn repository_error_is_wrapped() {
        let repo = MockProductRepository {
            fail: true,
            ..Default::default()
        };
        let service = ProductService::new(repo);

        let result = service.list().await;

        assert!(matches!(result, Err(MockError)));
    }

    #[tokio::test]
    async fn remove_product_success() {
        let repo = MockProductRepository::default();
        let service = ProductService::new(repo);

        let product = service.add("Temp".into(), "Temp".into(), 1).await.unwrap();
        let len_before = service.list().await.unwrap().len();

        let result = service.remove(product.id).await;
        assert!(result.is_ok());

        let len_after = service.list().await.unwrap().len();
        assert_ne!(len_before, len_after);
    }
}
