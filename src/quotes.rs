use quote::{format_ident, quote};

use crate::{
    attr::{Attr, PrimaryKey, ReturnObject},
    database,
};

#[derive(Debug, Clone)]
enum ReturnType {
    PrimaryKey(PrimaryKey),
    EntireRow(ReturnObject),
    OptionalRow(ReturnObject),
    MultipleRows(ReturnObject),
    None,
}

impl ReturnType {
    fn function_output(self) -> proc_macro2::TokenStream {
        match self {
            ReturnType::PrimaryKey(primary_key) => {
                let pk_type = primary_key._type;
                quote! {
                    sqlx::Result<#pk_type>
                }
            }
            ReturnType::EntireRow(return_object) => quote! {
                sqlx::Result<#return_object>
            },
            ReturnType::OptionalRow(return_object) => quote! {
                sqlx::Result<Option<#return_object>>
            },
            ReturnType::MultipleRows(return_object) => quote! {
                sqlx::Result<Vec<#return_object>>
            },
            ReturnType::None => quote! {
                sqlx::Result<()>
            },
        }
    }

    fn returning_statement(self) -> proc_macro2::TokenStream {
        match self {
            ReturnType::PrimaryKey(primary_key) => {
                let pk_name = primary_key.name;
                quote! {
                    qb.push(" RETURNING ");
                    qb.push(#pk_name);
                }
            }
            ReturnType::EntireRow(_) | ReturnType::OptionalRow(_) | ReturnType::MultipleRows(_) => {
                quote! {
                    qb.push(" RETURNING * ");
                }
            }
            ReturnType::None => quote! {},
        }
    }

    fn query_builder_execution(self) -> proc_macro2::TokenStream {
        match self {
            ReturnType::PrimaryKey(_) => quote! {
                qb.build()
                .fetch_one(db)
                .await
                .map(|row| row.get(0))
            },
            ReturnType::EntireRow(_) => quote! {
                qb.build_query_as()
                .fetch_one(db)
                .await
            },
            ReturnType::OptionalRow(_) => quote! {
                qb.build_query_as()
                .fetch_optional(db)
                .await
            },
            ReturnType::MultipleRows(_) => quote! {
                qb.build_query_as()
                .fetch_all(db)
                .await
            },
            ReturnType::None => quote! {
                qb.build()
                .execute(db)
                .await
                .map(|_| ())
            },
        }
    }
}

pub fn get_by_id_fn(attr: &Attr) -> proc_macro2::TokenStream {
    let db_type_ident = database::db_type().to_ident();
    let return_type = ReturnType::OptionalRow(attr.clone().return_object);
    let function_output = return_type.clone().function_output();
    let query_builder_execution = return_type.query_builder_execution();
    let table_name = attr.table_name.clone().to_string();

    let (pk_name, pk_type) = match attr.primary_key {
        Some(ref pk) => (&pk.name, &pk._type),
        None => panic!("No primary key field found which is mandatory for the 'get' operation"),
    };
    quote! {
        pub async fn get_by_id(db: &sqlx::Pool<sqlx::#db_type_ident>, id: #pk_type) -> #function_output {
            let mut qb = sqlx::QueryBuilder::new("SELECT * FROM ");
            qb.push(#table_name);
            qb.push(" WHERE ");
            qb.push(#pk_name);
            qb.push(" = ");
            qb.push_bind(id);

            #query_builder_execution
        }
    }
}

pub fn list_all_fn(attr: &Attr) -> proc_macro2::TokenStream {
    let db_type_ident = database::db_type().to_ident();
    let return_type = ReturnType::MultipleRows(attr.clone().return_object);
    let function_output = return_type.clone().function_output();
    let query_builder_execution = return_type.query_builder_execution();
    let table_name = attr.table_name.clone().to_string();

    quote! {
        pub async fn list_all(db: &sqlx::Pool<sqlx::#db_type_ident>) -> #function_output {
            let mut qb = sqlx::QueryBuilder::new("SELECT * FROM ");
            qb.push(#table_name);
            #query_builder_execution
        }
    }
}

pub fn create_fn(attr: &Attr) -> proc_macro2::TokenStream {
    let db_type_ident = database::db_type().to_ident();
    let table_name = attr.table_name.clone().to_string();

    let return_type = match attr.primary_key.clone() {
        None => ReturnType::EntireRow(attr.return_object.clone()),
        Some(primary_key) => ReturnType::PrimaryKey(primary_key),
    };

    let function_output = return_type.clone().function_output();
    let returning_statement = return_type.clone().returning_statement();
    let query_builder_execution = return_type.query_builder_execution();

    let field_idents: Vec<syn::Ident> = attr
        .field_names
        .iter()
        .map(|f| format_ident!("{}", f))
        .collect();
    let fields_str = attr.field_names.join(", ");

    quote! {
        pub async fn create(&self, db: &sqlx::Pool<sqlx::#db_type_ident>) -> #function_output {
            let mut qb = sqlx::QueryBuilder::new("INSERT INTO ");
            qb.push(#table_name);
            qb.push(" (");
            qb.push(#fields_str);
            qb.push(") VALUES (");

            let mut separated = qb.separated(", ");
            #(
                separated.push_bind(&self.#field_idents);
            )*
            separated.push_unseparated(")");

            #returning_statement

            #query_builder_execution
        }
    }
}

pub fn update_fn(attr: &Attr) -> proc_macro2::TokenStream {
    let db_type_ident = database::db_type().to_ident();
    let return_type = ReturnType::EntireRow(attr.return_object.clone());
    let function_output = return_type.clone().function_output();
    let query_builder_execution = return_type.clone().query_builder_execution();
    let returning_statement = return_type.returning_statement();

    let table_name = attr.table_name.clone().to_string();
    let (pk_name, pk_ident) = match attr.primary_key {
        Some(ref pk) => (&pk.name, &pk.ident),
        None => panic!("No primary key field found"),
    };
    let update_fields = attr
        .field_names
        .clone()
        .into_iter()
        .filter(|f| f != pk_name)
        .collect::<Vec<_>>();

    let update_field_idents: Vec<syn::Ident> = update_fields
        .iter()
        .map(|f| format_ident!("{}", f))
        .collect::<Vec<_>>();

    quote! {
        pub async fn update(&self, db: &sqlx::Pool<sqlx::#db_type_ident>) -> #function_output {
            let mut qb = sqlx::QueryBuilder::new("UPDATE ");
            qb.push(#table_name);
            qb.push(" SET ");

            let mut first = true;
            #(
                if !first {
                    qb.push(", ");
                }
                qb.push(#update_fields);
                qb.push(" = ");
                qb.push_bind(&self.#update_field_idents);
                first = false;
            )*

            qb.push(" WHERE ");
            qb.push(#pk_name);
            qb.push(" = ");
            qb.push_bind(&self.#pk_ident);

            #returning_statement

            #query_builder_execution
        }
    }
}

pub fn delete_fn(attr: &Attr) -> proc_macro2::TokenStream {
    let db_type_ident = database::db_type().to_ident();
    let return_type = ReturnType::None;
    let function_output = return_type.clone().function_output();
    let query_builder_execution = return_type.query_builder_execution();
    let table_name = attr.table_name.clone().to_string();
    let (pk_name, pk_ident) = match attr.primary_key {
        Some(ref pk) => (&pk.name, &pk.ident),
        None => panic!("No primary key field found"),
    };
    quote! {
        pub async fn delete(&self, db: &sqlx::Pool<sqlx::#db_type_ident>) -> #function_output {
            let mut qb = sqlx::QueryBuilder::new("DELETE FROM ");
            qb.push(#table_name);
            qb.push(" WHERE ");
            qb.push(#pk_name);
            qb.push(" = ");
            qb.push_bind(&self.#pk_ident);

            #query_builder_execution
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn clean_tokens(tokens: proc_macro2::TokenStream) -> String {
        tokens.to_string().replace([' ', '\n'], "")
    }

    mod simple_attr {
        use quote::format_ident;
        use syn::parse_quote;

        use crate::attr::{Column, Operation, TableName};

        use super::*;

        fn input() -> Attr {
            Attr {
                struct_name: format_ident!("Contact"),
                table_name: TableName::new("contact".to_string()),
                return_object: format_ident!("Self"),
                primary_key: Some(Column::new("id".to_string(), parse_quote!(i64))),
                created_at: Some(Column::new(
                    "created_at".to_string(),
                    parse_quote!(DateTime<Utc>),
                )),
                updated_at: Some(Column::new(
                    "updated_at".to_string(),
                    parse_quote!(DateTime<Utc>),
                )),
                field_names: vec![
                    "id".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                    "last_name".to_string(),
                ],
                operations: Operation::all(),
            }
        }

        #[test]
        fn test_generate_create_method() {
            let generated = clean_tokens(create_fn(&input()));

            let expected = clean_tokens(quote! {
                pub async fn create(&self, db: &sqlx::Pool<sqlx::Sqlite>) -> sqlx::Result<i64>{
                    let mut qb = sqlx::QueryBuilder::new("INSERT INTO ");
                    qb.push("contact");
                    qb.push(" (");
                    qb.push("id, created_at, updated_at, last_name");
                    qb.push(") VALUES (");

                    let mut separated = qb.separated(", ");
                    separated.push_bind(&self.id);
                    separated.push_bind(&self.created_at);
                    separated.push_bind(&self.updated_at);
                    separated.push_bind(&self.last_name);
                    separated.push_unseparated(")");

                    qb.push(" RETURNING ");
                    qb.push("id");

                    qb.build()
                    .fetch_one(db)
                    .await
                    .map(|row|row.get(0))
                }
            });

            assert_eq!(generated, expected);
        }

        #[test]
        fn test_generate_get_by_id_method() {
            let generated = clean_tokens(get_by_id_fn(&input()));

            let expected = clean_tokens(quote! {
                pub async fn get_by_id(db: &sqlx::Pool<sqlx::Sqlite>, id: i64) -> sqlx::Result<Option<Self>> {
                    let mut qb = sqlx::QueryBuilder::new("SELECT * FROM ");
                    qb.push("contact");
                    qb.push(" WHERE ");
                    qb.push("id");
                    qb.push(" = ");
                    qb.push_bind(id);

                    qb.build_query_as()
                    .fetch_optional(db)
                    .await
                }
            });

            assert_eq!(generated, expected);
        }

        #[test]
        fn test_generate_list_all_method() {
            let generated = clean_tokens(list_all_fn(&input()));

            let expected = clean_tokens(quote! {
                pub async fn list_all(db: &sqlx::Pool<sqlx::Sqlite>) -> sqlx::Result<Vec<Self>> {
                    let mut qb = sqlx::QueryBuilder::new("SELECT * FROM ");
                    qb.push("contact");

                    qb.build_query_as()
                    .fetch_all(db)
                    .await
                }
            });

            assert_eq!(generated, expected);
        }

        #[test]
        fn test_generate_update_method() {
            let generated = clean_tokens(update_fn(&input()));

            let expected = clean_tokens(quote! {
                pub async fn update(&self, db: &sqlx::Pool<sqlx::Sqlite>) -> sqlx::Result<Self> {
                    let mut qb = sqlx::QueryBuilder::new("UPDATE ");
                    qb.push("contact");
                    qb.push(" SET ");

                    let mut first = true;

                    if !first {
                        qb.push(",");
                    }
                    qb.push("created_at");
                    qb.push(" = ");
                    qb.push_bind(&self.created_at);
                    first = false;

                    if !first {
                        qb.push(",");
                    }
                    qb.push("updated_at");
                    qb.push(" = ");
                    qb.push_bind(&self.updated_at);
                    first = false;

                    if !first {
                        qb.push(",");
                    }
                    qb.push("last_name");
                    qb.push(" = ");
                    qb.push_bind(&self.last_name);
                    first = false;

                    qb.push(" WHERE ");
                    qb.push("id");
                    qb.push(" = ");
                    qb.push_bind(&self.id);

                    qb.push(" RETURNING * ");

                    qb.build_query_as()
                    .fetch_one(db)
                    .await
                }
            });

            assert_eq!(generated, expected);
        }

        #[test]
        fn test_generate_delete_method() {
            let generated = clean_tokens(delete_fn(&input()));

            let expected = clean_tokens(quote! {
                pub async fn delete(&self, db: &sqlx::Pool<sqlx::Sqlite>) -> sqlx::Result<()> {
                    let mut qb = sqlx::QueryBuilder::new("DELETE FROM ");
                    qb.push("contact");
                    qb.push(" WHERE ");
                    qb.push("id");
                    qb.push(" = ");
                    qb.push_bind(&self.id);

                    qb.build()
                    .execute(db)
                    .await
                    .map(|_| ())
                }
            });

            assert_eq!(generated, expected);
        }
    }

    mod custom {
        use syn::parse_quote;

        use super::*;
        use crate::attr::*;

        #[test]
        fn test_custom_output_create() {
            let input = Attr {
                struct_name: format_ident!("NewContact"),
                table_name: TableName::new("contact".to_string()),
                return_object: format_ident!("Contact"),
                primary_key: None,
                created_at: None,
                updated_at: None,
                field_names: vec![
                    "first_name".to_string(),
                    "last_name".to_string(),
                    "email".to_string(),
                ],
                operations: vec![Operation::Create],
            };

            let generated = clean_tokens(create_fn(&input));

            let expected = clean_tokens(quote! {
                pub async fn create(&self, db: &sqlx::Pool<sqlx::Sqlite>) -> sqlx::Result<Contact>{
                    let mut qb = sqlx::QueryBuilder::new("INSERT INTO ");
                    qb.push("contact");
                    qb.push(" (");
                    qb.push("first_name, last_name, email");
                    qb.push(") VALUES (");

                    let mut separated = qb.separated(", ");
                    separated.push_bind(&self.first_name);
                    separated.push_bind(&self.last_name);
                    separated.push_bind(&self.email);
                    separated.push_unseparated(")");

                    qb.push(" RETURNING * ");

                    qb.build_query_as()
                    .fetch_one(db)
                    .await
                }
            });
            assert_eq!(generated, expected);
        }

        #[test]
        fn test_custom_output_update() {
            let input = Attr {
                struct_name: format_ident!("UpdateContact"),
                table_name: TableName::new("contact".to_string()),
                return_object: format_ident!("Contact"),
                primary_key: Some(Column::new("custom_id".to_string(), parse_quote!(i64))),
                created_at: None,
                updated_at: None,
                field_names: vec!["custom_id".to_string(), "first_name".to_string()],
                operations: vec![Operation::Update],
            };

            let generated = clean_tokens(update_fn(&input));

            let expected = clean_tokens(quote! {
                pub async fn update(&self, db: &sqlx::Pool<sqlx::Sqlite>) -> sqlx::Result<Contact> {
                    let mut qb = sqlx::QueryBuilder::new("UPDATE ");
                    qb.push("contact");
                    qb.push(" SET ");

                    let mut first = true;

                    if !first {
                        qb.push(",");
                    }
                    qb.push("first_name");
                    qb.push(" = ");
                    qb.push_bind(&self.first_name);
                    first = false;

                    qb.push(" WHERE ");
                    qb.push("custom_id");
                    qb.push(" = ");
                    qb.push_bind(&self.custom_id);

                    qb.push(" RETURNING * ");

                    qb.build_query_as()
                    .fetch_one(db)
                    .await
                }
            });

            assert_eq!(generated, expected);
        }
    }
}