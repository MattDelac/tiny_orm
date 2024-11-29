# TODOs Example

## Setup

```
$ docker run --rm -p 5432:5432 -e POSTGRES_PASSWORD=password postgres
$ sqlx db reset --database-url "postgres://postgres:password@localhost/examples"
```

## Usage

```
cargo run
```
