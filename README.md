[![Crates.io License](https://img.shields.io/crates/l/tiny-orm)](./LICENSE)
[![ci](https://github.com/MattDelac/tiny_orm/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/MattDelac/tiny_orm/actions/workflows/ci.yaml)
[![Publish](https://github.com/MattDelac/tiny_orm/actions/workflows/publish.yaml/badge.svg)](https://github.com/MattDelac/tiny_orm/actions/workflows/publish.yaml)
[![GitHub Repo stars](https://img.shields.io/github/stars/mattdelac/tiny_orm)](https://github.com/MattDelac/tiny_orm)
[![Crates.io Version](https://img.shields.io/crates/v/tiny-orm)](https://crates.io/crates/tiny-orm)
[![Crates.io MSRV](https://img.shields.io/crates/msrv/tiny-orm)](https://crates.io/crates/tiny-orm)
[![Crates.io Size](https://img.shields.io/crates/size/tiny-orm)](https://crates.io/crates/tiny-orm)
[![docs.rs](https://img.shields.io/docsrs/tiny-orm)](https://docs.rs/tiny-orm/0.2.0/tiny_orm/)


# Tiny ORM

A minimal ORM for CRUD operations. Built on top of SQLx and with its QueryBuilder. Support Sqlite, Postgres and MySQL right off the bat. It has smart defaults and is also flexible.

This library is the one I wished I had when I built a project in Rust in production and did not want a heavy ORM.

## Installation
```sh
cargo install tiny-orm -F postgres # sqlite or mysql
```

Or add this to your `Cargo.toml`:
```toml
[dependencies]
tiny-orm = {version = "0.2.0", features = ["postgres"] } # Choose between sqlite, mysql and postgres
```

## Principles & advantages of TinyORM
The goals of this library are
- To have the least amount of dependencies
- Tackle all the boilerplate of the CRUD operations
- Add no complexity by default
- Can fit real world problems

Why TinyORM over another one?
- All the queries are built with the QueryBuilder  
-- All the inputs are passed as database arguments  
-- All the queries are compatible with Sqlite, Postgres and MySQL

- Minimal set of dependencies (fast compile time)  
-- [SQLx](https://github.com/launchbadge/sqlx)  
-- [convert_case](https://github.com/rutrum/convert-case)  
-- [regex](https://crates.io/crates/regex)  
-- [lazy_static](https://crates.io/crates/https://crates.io/crates/lazy_static)  

- Intuitive with smart defaults and flexible

### MSRV
MSRV has been tested with `cargo-msrv find --features sqlite`. You can learn more about [cargo-msrv on their website](https://gribnau.dev/cargo-msrv/getting-started/quick-start.html).  
Latest check ran on 0.2.0.

## License
This project is licensed under [MIT] - see the [LICENSE](LICENSE) file for details.

## Usage
<!-- cargo-rdme start -->

```rust
use sqlx::{FromRow, Row, types::chrono::{DateTime, Utc}};
use tiny_orm::Table;

#[derive(Debug, FromRow, Table, Clone)]
#[tiny_orm(all)]
struct Todo {
    id: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: String,
    done: bool,
}
```

The code above would generate the following methods on the Todo object
```rust
impl Todo {
    pub fn get_by_id(pool: &DbPool, id: &i32) -> sqlx::Result<Self> {
        // Get a specific record for a given ID
        // Use the `id` column by default
    }
    pub fn list_all(pool: &DbPool) -> sqlx::Result<Vec<Self>> {
        // Get all the records
    }
    pub fn delete(&self, pool: &DbPool) -> sqlx::Result<()> {
        // Delete the record in the database
    }
    pub fn create(&self, pool: &DbPool) -> sqlx::Result<i32> {
        // Create the Todo object as a record in
        // the database and returns the primary key of the record created.
    }
    pub fn update(&self, pool: &DbPool) -> sqlx::Result<()> {
        // Update the record in the database with the values
        // currently part of the Todo object
    }
}
```

### Examples
More examples can be found in the [examples](./examples) directory.

#### Options
The Table macro comes with a few options to give flexibility to the user

##### At the Struct level
- **table_name**: The name of the table in the database.
  Default being a lower_case version of the Struct name. So `MyStruct` would have `my_struct` as a default `table_name`.
- **only**: The methods that will only be available to that struct. Multiple values are comma separated.
  Default is dependent on the struct name (see below).
- **exclude**: The methods that will be excluded for that struct. Multiple values are comma separated
  Default would be nothing.
- **all**: All the methods will be available to the struct. This will override the default values when none are provided.
  Default none.
- **return_object**: A custom object that would be returned instead of `Self`. Useful when creating or updating records with partial information.
  Default is `Self` which corresponds to the current Strut.

_Note: `only` and `exclude` cannot be used together._

By convention, if a struct name
- Starts with `New` then it will automatically use the following arguments
  `#[tiny_orm(table_name = "table_name_from_return_object", return_object = "NameWithoutNewAsPrefix", only = "create")]`
- Starts with `Update` then it will automatically use the following arguments
  `#[tiny_orm(table_name = "table_name_from_return_object", return_object = "NameWithoutUpdateAsPrefix", only = "update")]`
- Everything else would be the equivalent of using the following
  `#[tiny_orm(table_name = "table_name", return_object = "Self", exclude = "create,update")]`

Example
```rust
#[derive(Debug, FromRow, Table, Clone)]
// #[tiny_orm(table_name = "todo", return_object = "Self", exclude = "create,update")]
struct Todo {
    id: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: String,
    done: bool,
}

#[derive(Debug, FromRow, Table, Clone)]
// Because the struct name starts with "New", it would automatically use the following options
// #[tiny_orm(table_name = "todo", return_object = "Todo", only = "create")]
struct NewTodo {
    description: String,
}

#[derive(Debug, FromRow, Table, Clone)]
// Because the struct name starts with "Update", it would automatically use the following options
// #[tiny_orm(table_name = "todo", return_object = "Todo", only = "update")]
struct UpdateTodo {
    id: i32,
    done: bool
}
```
The above takes the assumption that some columns can be nullable and others are auto generated (eg an ID column that would be auto increment).
Thus it would generate the following
```rust
impl NewTodo {
    pub fn create(&self, pool: &DbPool) -> sqlx::Result<Todo> {
        // Use the NewTodo object to create a record
        // in the database and return the record created.

        // This would not be supported for MySQL since
        // there is a need to know the primary key (auto increment or not)
        // to either return the value or use `last_insert_id()`
    }
}

impl UpdateTodo {
    pub fn update(&self, pool: &DbPool) -> sqlx::Result<Todo> {
        // Update the Todo object with partial information based
        // on the id (default primary_key).
        // Because `return_type` is specified, it will return the whole record in the database.
        // If `return_type` was not specified, it would simply return nothing `()`.

        // For MySQL, it would always return nothing `()` as it
        // cannot return a record after an update.
    }
}
```

##### At the field level
- **primary_key**: The field that would be used as a primary key for the queries. For some methods the primary key is mandatory (eg: `get_by_id()`). If not specified, it will default to `id` if part of the struct.
- **primary_key(auto)**: The field that would be used as a primary key for the queries. When "auto" is set, it will skip the column and returns it during the `create()` method since it will be auto generated by the database itself.

_Note: MySQL only supports "auto increment" in that case. It does not support returning the default value of a primary key like a UUID._

Example
```rust
#[derive(Debug, FromRow, Table, Clone)]
struct Todo {
    #[tiny_orm(primary_key)]
    custom_pk: Uuid,
    description: String,
    done: bool
}
```

```rust
use tiny_orm::{Table, SetOption};
#[derive(Debug, FromRow, Table, Clone)]
struct Todo {
    #[tiny_orm(primary_key(auto))]
    id: i64,
    description: SetOption<String>,
    done: SetOption<bool>
}
```

Thus it would generate the following
```rust
impl Todo {
    pub fn create(&self, pool: &DbPool) -> sqlx::Result<Uuid> { // or i64 depending on the example
        // Create the Todo object as a record in
        // the database and returns the created record.
    }
}
```

<!-- cargo-rdme end -->

## Migrations
### From 0.1.* to 0.2.*
In 0.2.0, the API was stabilized with the fact that
- The `update()` and `delete()` methods now always return an empty Ok response `sqlx::Result<()>`
- The `create()` method always returns the primary key except when the return type is of another object (eg: `NewTodo` has a return_object of `Todo`)

## Roadmap
Goal of TinyORM is to stay tiny. That being said there are still a few things I would like to have
- [X] Get all the CRUD methods.
- [X] Ability to have custom fields like a custom primary_key or table_name.
- [X] Ability to choose the operations available on the method through an opt-in strategy or an exclusion one.
- [X] Ability to return a custom object type for more complex setup.
- [X] Support MySQL with non auto increment PK. Won't be able to use the `last_return_id()` function in the case (eg with UUID).
- [ ] Auto generate the `created_at` / `updated_at` column based on some default value when needed.  
-- For example, being able to have `Utc.now()` on the `update()` method for the `updated_at` if the field is of type `DateTime<Utc>`
- [ ] Ability to skip some fields if not set (especially for the create and update methods).  
-- For example if the field is of type `SetOption<DateTime<Utc>>` then it would only put the field as part of the query if the value is `Set(value)`. It would skip it if it has the value `NotSet`.

## Release
One PR with the following
```sh
make release VERSION=v0.1.2
```

then add the tag and push it to main
```sh
git tag -a v0.1.2 -m "Release v0.1.2"
git push origin main --tags
```
