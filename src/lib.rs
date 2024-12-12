//! ```rust
//! use sqlx_tiny_orm::Table;
//!
//! #[derive(Debug, FromRow, Table, Clone)]
//! #[tiny_orm(all)]
//! struct Todo {
//!     id: i32,
//!     created_at: DateTime<Utc>,
//!     updated_at: DateTime<Utc>,
//!     description: String,
//!     done: bool,
//! }
//! ```
//!
//! The code above would generate the following methods on the Todo object
//! ```rust
//! impl Todo {
//!     pub fn get_by_id(pool: &DbPool, id: &i32) -> sqlx::Result<Self> {
//!         // Get a specific record for a given ID
//!         // Use the `id` column by default
//!     }
//!     pub fn list_all(pool: &DbPool) -> sqlx::Result<Vec<Self>> {
//!         // Get all the records
//!     }
//!     pub fn delete(&self, pool: &DbPool) -> sqlx::Result<()> {
//!         // Delete the record in the database
//!     }
//!     pub fn create(&self, pool: &DbPool) -> sqlx::Result<i32> {
//!         // Create the Todo object as a record in
//!         // the database and returns the primary key of the record created.
//!     }
//!     pub fn update(&self, pool: &DbPool) -> sqlx::Result<()> {
//!         // Update the record in the database with the values
//!         // currently part of the Todo object
//!     }
//! }
//! ```
//!
//! # Examples
//! More examples can be found in the [examples](./examples) directory.
//!
//! ## Options
//! The Table macro comes with a few options to give flexibility to the user
//!
//! ### At the Struct level
//! - **table_name**: The name of the table in the database.
//!   Default being a lower_case version of the Struct name. So `MyStruct` would have `my_struct` as a default `table_name`.
//! - **only**: The methods that will only be available to that struct. Multiple values are comma separated.
//!   Default is dependent on the struct name (see below).
//! - **exclude**: The methods that will be excluded for that struct. Multiple values are comma separated
//!   Default would be nothing.
//! - **all**: All the methods will be available to the struct. This will override the default values when none are provided.
//!   Default none.
//! - **return_object**: A custom object that would be returned instead of `Self`. Useful when creating or updating records with partial information.
//!   Default is `Self` which corresponds to the current Strut.
//!
//! _Note: `only` and `exclude` cannot be used together._
//!
//! By convention, if a struct name
//! - Starts with `New` then it will automatically use the following arguments
//!   `#[tiny_orm(table_name = "table_name_from_return_object", return_object = "NameWithoutNewAsPrefix", only = "create")]`
//! - Starts with `Update` then it will automatically use the following arguments
//!   `#[tiny_orm(table_name = "table_name_from_return_object", return_object = "NameWithoutUpdateAsPrefix", only = "update")]`
//! - Everything else would be the equivalent of using the following
//!   `#[tiny_orm(table_name = "table_name", return_object = "Self", exclude = "create,update")]`
//!
//! Example
//! ```rust
//! #[derive(Debug, FromRow, Table, Clone)]
//! // #[tiny_orm(table_name = "todo", return_object = "Self", exclude = "create,update")]
//! struct Todo {
//!     id: i32,
//!     created_at: DateTime<Utc>,
//!     updated_at: DateTime<Utc>,
//!     description: String,
//!     done: bool,
//! }
//!
//! #[derive(Debug, FromRow, Table, Clone)]
//! // Because the struct name starts with "New", it would automatically use the following options
//! // #[tiny_orm(table_name = "todo", return_object = "Todo", only = "create")]
//! struct NewTodo {
//!     description: String,
//! }
//!
//! #[derive(Debug, FromRow, Table, Clone)]
//! // Because the struct name starts with "Update", it would automatically use the following options
//! // #[tiny_orm(table_name = "todo", return_object = "Todo", only = "update")]
//! struct UpdateTodo {
//!     id: i32,
//!     done: bool
//! }
//! ```
//! The above takes the assumption that some columns can be nullable and others are auto generated (eg an ID column that would be auto increment).
//! Thus it would generate the following
//! ```rust
//! impl NewTodo {
//!     pub fn create(&self, pool: &DbPool) -> sqlx::Result<Todo> {
//!         // Use the NewTodo object to create a record
//!         // in the database and return the record created.
//!
//!         // This would not be supported for MySQL since
//!         // there is a need to know the primary key (auto increment or not)
//!         // to either return the value or use `last_insert_id()`
//!     }
//! }
//!
//! impl UpdateTodo {
//!     pub fn update(&self, pool: &DbPool) -> sqlx::Result<Todo> {
//!         // Update the Todo object with partial information based
//!         // on the id (default primary_key).
//!         // Because `return_type` is specified, it will return the whole record in the database.
//!         // If `return_type` was not specified, it would simply return nothing `()`.
//!
//!         // For MySQL, it would always return nothing `()` as it
//!         // cannot return a record after an update.
//!     }
//! }
//! ```
//!
//! ### At the field level
//! - **primary_key**: The field that would be used as a primary key for the queries. For some methods the primary key is mandatory (eg: `get_by_id()`). If not specified, it will default to `id` if part of the struct.
//! - **primary_key(auto)**: The field that would be used as a primary key for the queries. When "auto" is set, it will skip the column and returns it during the `create()` method since it will be auto generated by the database itself.
//!
//! _Note: MySQL only supports "auto increment" in that case. It does not support returning the default value of a primary key like a UUID._
//!
//! Example
//! ```rust
//! #[derive(Debug, FromRow, Table, Clone)]
//! struct Todo {
//!     #[tiny_orm(primary_key)]
//!     custom_pk: Uuid,
//!     description: String,
//!     done: bool
//! }
//! ```
//!
//! ```rust
//! #[derive(Debug, FromRow, Table, Clone)]
//! struct Todo {
//!     #[tiny_orm(primary_key(auto))]
//!     id: i64,
//!     description: String,
//!     done: bool
//! }
//! ```
//!
//! Thus it would generate the following
//! ```rust
//! impl Todo {
//!     pub fn create(&self, pool: &DbPool) -> sqlx::Result<Uuid> { // or i64 depending on the example
//!         // Create the Todo object as a record in
//!         // the database and returns the created record.
//!     }
//! }
//! ```

#![allow(dead_code)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use types::Operation;

mod attr;
mod database;
mod quotes;
mod types;

#[proc_macro_derive(Table, attributes(tiny_orm))]
pub fn derive_tiny_orm(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let attr = attr::Attr::parse(input);

    let expanded = generate_impl(&attr);

    #[cfg(test)]
    println!("Generated code:\n{}", expanded);

    TokenStream::from(expanded)
}

fn generate_impl(attr: &attr::Attr) -> proc_macro2::TokenStream {
    let struct_name = attr.parsed_struct.name.clone();

    let get_impl = if attr.operations.contains(&Operation::Get) {
        quotes::get_by_id_fn(attr)
    } else {
        quote! {}
    };

    let list_impl = if attr.operations.contains(&Operation::List) {
        quotes::list_all_fn(attr)
    } else {
        quote! {}
    };

    let create_impl = if attr.operations.contains(&Operation::Create) {
        quotes::create_fn(attr)
    } else {
        quote! {}
    };

    let update_impl = if attr.operations.contains(&Operation::Update) {
        quotes::update_fn(attr)
    } else {
        quote! {}
    };

    let delete_impl = if attr.operations.contains(&Operation::Delete) {
        quotes::delete_fn(attr)
    } else {
        quote! {}
    };

    quote! {
        impl #struct_name {
            #get_impl
            #list_impl
            #create_impl
            #update_impl
            #delete_impl
        }
    }
}
