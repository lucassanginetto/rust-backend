# Rust Backend

A simple CRUD API written in Rust, trying to follow Clean Architecture principles. It uses the [Actix Web](https://actix.rs/) framework, and the `sqlx` toolkit for handling connections to the PostgreSQL database.

## Running

```sh
# Create dotenv file using the example
cp .env.example .env

# Run PostgreSQL Docker container
docker-compose up -d

# Run migrations
sqlx migrate run

# Run API
cargo run
```
