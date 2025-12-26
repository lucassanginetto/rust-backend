use actix_web::{HttpResponse, http::header::LOCATION, web};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    application::product_service::{ProductRepository, ProductService, ProductServiceError},
    domain::product::Product,
};

#[derive(Deserialize)]
pub struct CreateProductDTO {
    pub name: String,
    pub description: String,
    pub price: u32,
}
#[derive(Serialize)]
pub struct OutputProductDTO {
    id: Uuid,
    name: String,
    description: String,
    price: u32,
}
impl From<Product> for OutputProductDTO {
    fn from(value: Product) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            price: value.price,
        }
    }
}

pub async fn list_products<R: ProductRepository>(
    service: web::Data<ProductService<R>>,
) -> HttpResponse {
    match service.list().await {
        Ok(products) => HttpResponse::Ok().json(
            products
                .into_iter()
                .map(|product| OutputProductDTO::from(product))
                .collect::<Vec<_>>(),
        ),
        Err(error) => {
            log::error!("error while listing products: {}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn add_product<R: ProductRepository>(
    service: web::Data<ProductService<R>>,
    payload: web::Json<CreateProductDTO>,
) -> HttpResponse {
    let dto = payload.into_inner();
    match service.add(dto.name, dto.description, dto.price).await {
        Ok(product) => HttpResponse::Created()
            .insert_header((LOCATION, format!("/api/products/{}", product.id)))
            .json(OutputProductDTO::from(product)),
        Err(error) => {
            log::error!("error while creating product: {}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn find_product<R: ProductRepository>(
    service: web::Data<ProductService<R>>,
    id: web::Path<Uuid>,
) -> HttpResponse {
    match service.find(id.into_inner()).await {
        Ok(product) => HttpResponse::Ok().json(OutputProductDTO::from(product)),
        Err(ProductServiceError::NotFound) => HttpResponse::NotFound().finish(),
        Err(ProductServiceError::Repository(error)) => {
            log::error!("error while getting product: {}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn put_product<R: ProductRepository>(
    service: web::Data<ProductService<R>>,
    id: web::Path<Uuid>,
    payload: web::Json<CreateProductDTO>,
) -> HttpResponse {
    let dto = payload.into_inner();
    match service
        .modify(id.into_inner(), dto.name, dto.description, dto.price)
        .await
    {
        Ok(product) => HttpResponse::Ok().json(OutputProductDTO::from(product)),
        Err(ProductServiceError::NotFound) => HttpResponse::NotFound().finish(),
        Err(ProductServiceError::Repository(error)) => {
            log::error!("error while modifying product: {}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn remove_product<R: ProductRepository>(
    service: web::Data<ProductService<R>>,
    id: web::Path<Uuid>,
) -> HttpResponse {
    match service.remove(id.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(ProductServiceError::NotFound) => HttpResponse::NotFound().finish(),
        Err(ProductServiceError::Repository(error)) => {
            log::error!("error while deleting product: {}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}
