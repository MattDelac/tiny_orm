# TODOs Example

## Usage

```sh
cd examples/postgres
docker run --rm -p 5432:5432 -e POSTGRES_PASSWORD=password postgres
sqlx db setup --database-url "postgres://postgres:password@localhost/examples"
cargo run --example postgres --features postgres
```
