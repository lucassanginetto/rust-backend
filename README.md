# Rust Backend

A simple CRUD API written in Rust, using the [Actix Web](https://actix.rs/) framework. It connects to a PostgreSQL database (through `sqlx`) for storing it's data, and to a Redis database for caching. Both databases can be run easily using the provided Docker Compose file.

## Running

```sh
# Create dotenv file using the example
cp .env.example .env

# Run PostgreSQL and Redis Docker containers
docker-compose up -d

# Run migrations
sqlx migrate run

# Run API
cargo run
```
