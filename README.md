[![Crates.io License](https://img.shields.io/crates/l/tiny-orm)](./LICENSE)
[![ci](https://github.com/MattDelac/tiny_orm/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/MattDelac/tiny_orm/actions/workflows/ci.yaml)
[![Publish](https://github.com/MattDelac/tiny_orm/actions/workflows/publish.yaml/badge.svg)](https://github.com/MattDelac/tiny_orm/actions/workflows/publish.yaml)
[![Crates.io Version](https://img.shields.io/crates/v/tiny-orm)](https://crates.io/crates/tiny-orm)


# Tiny ORM

A minimal ORM for CRUD operations. Built on top of SQLx and with its QueryBuilder. Support Sqlite, Postgres and MySQL right off the bat. It has smart defaults and is also flexible.

This library is the one I wished I had when I built a project in Rust in production and did not want a heavy ORM.

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
-- Standard libraries for a proc macro (syn, etc.)  
-- [convert_case](https://github.com/rutrum/convert-case)

- Intuitive with smart defaults and flexible

## Installation
Add this to your `Cargo.toml`:
```toml
[dependencies]
tiny-orm = {version = "0.1.4", features = ["postgres"] } # Choose between sqlite, mysql and postgres
```

## License
This project is licensed under [MIT] - see the [LICENSE](LICENSE) file for details.

## Usage
### Basic
```rust
use sqlx_tiny_orm::Table;

#[derive(Debug, FromRow, Table, Clone)]
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

## Examples
More examples can be found in the [examples](./examples) directory.

### Options
The Table macro comes with a few options to give flexibility to the user

#### At the Struct level
- **table_name**: The name of the table in the database.
Default being a lower_case version of the Struct name. So `MyStruct` would have `my_struct` as a default `table_name`.
- **only**: The methods that will only be available to that struct. Multiple values are comma separated
Default would be the equivalent of `only = "created,get,list,update,delete"`
- **exclude**: The methods that will be excluded for that struct. Multiple values are comma separated
Default would be nothing.
- **return_object**: A custom object that would be returned instead of `Self`. Useful when creating or updating records with partial information.
Default is `Self` which corresponds to the current Strut.

_Note: `only` and `exclude` cannot be used together._

Example
```rust
#[derive(Debug, FromRow, Table, Clone)]
#[tiny_orm(exclude = "create,update")]
struct Todo {
    id: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: String,
    done: bool,
}

#[derive(Debug, FromRow, Table, Clone)]
#[tiny_orm(table_name = "todo", return_object = "Todo", only = "create")]
struct NewTodos {
    description: String,
}

#[derive(Debug, FromRow, Table, Clone)]
#[tiny_orm(table_name = "todo", return_object = "Todo", only = "update")]
struct UpdateTodos {
    id: i32,
    done: bool
}
```
The above takes the assumption that some columns can be nullable and others are auto generated (eg an ID column that would be auto increment).
Thus it would generate the following
```rust
impl NewTodos {
    pub fn create(&self, pool: &DbPool) -> sqlx::Result<Todo> {
        // Use the NewTodo object to create a record
        // in the database and return the record created.

        // This would not be supported for MySQL since
        // there is a need to know the primary key (auto increment or not)
        // to either return the value or use `last_insert_id()`
    }
}

impl UpdateTodos {
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

#### At the field level
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
#[derive(Debug, FromRow, Table, Clone)]
struct Todo {
    #[tiny_orm(primary_key(auto))]
    id: i64,
    description: String,
    done: bool
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
