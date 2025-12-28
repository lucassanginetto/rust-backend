use actix_web::{App, web};
use uuid::Uuid;

use rust_backend::{
    application::product_service::{ProductRepository, ProductService},
    domain::product::Product,
};

#[derive(Default)]
struct MockProductRepository {
    products: std::sync::Mutex<Vec<Product>>,
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
        Ok(self.products.lock().unwrap().clone())
    }

    async fn read_one(&self, id: Uuid) -> Result<Option<Product>, Self::Error> {
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
        let mut products = self.products.lock().unwrap();
        let len_before = products.len();
        products.retain(|p| p.id != id);

        Ok(products.len() != len_before)
    }
}

fn test_app() -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    type Repo = MockProductRepository;
    let repo = Repo::default();
    let service = ProductService::new(repo);

    App::new().app_data(web::Data::new(service)).service(
        web::scope("/api/products")
            .route(
                "",
                web::get().to(rust_backend::handlers::product_handlers::list_products::<Repo>),
            )
            .route(
                "",
                web::post().to(rust_backend::handlers::product_handlers::add_product::<Repo>),
            )
            .route(
                "/{id}",
                web::get().to(rust_backend::handlers::product_handlers::find_product::<Repo>),
            )
            .route(
                "/{id}",
                web::delete().to(rust_backend::handlers::product_handlers::remove_product::<Repo>),
            ),
    )
}

#[actix_web::test]
async fn list_products_returns_200() {
    let app = actix_web::test::init_service(test_app()).await;

    let req = actix_web::test::TestRequest::get()
        .uri("/api/products")
        .to_request();

    let resp = actix_web::test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn add_product_returns_201() {
    let app = actix_web::test::init_service(test_app()).await;

    let payload = serde_json::json!({
        "name": "Book",
        "description": "A nice book",
        "price": 100
    });

    let req = actix_web::test::TestRequest::post()
        .uri("/api/products")
        .set_json(&payload)
        .to_request();

    let resp = actix_web::test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
}

#[actix_web::test]
async fn find_product_returns_404() {
    let app = actix_web::test::init_service(test_app()).await;

    let req = actix_web::test::TestRequest::get()
        .uri(&format!("/api/products/{}", Uuid::new_v4()))
        .to_request();

    let resp = actix_web::test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn delete_product_return_204() {
    let app = actix_web::test::init_service(test_app()).await;

    let payload = serde_json::json!({
        "name": "Temp",
        "description": "Temp",
        "price": 1
    });

    let create_req = actix_web::test::TestRequest::post()
        .uri("/api/products")
        .set_json(&payload)
        .to_request();

    let create_resp: serde_json::Value =
        actix_web::test::call_and_read_body_json(&app, create_req).await;
    let id = create_resp["id"].as_str().unwrap();

    let delete_req = actix_web::test::TestRequest::delete()
        .uri(&format!("/api/products/{}", id))
        .to_request();

    let delete_resp = actix_web::test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);
}
