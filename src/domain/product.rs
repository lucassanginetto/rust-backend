use uuid::Uuid;

#[derive(Clone)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price: u32,
}
