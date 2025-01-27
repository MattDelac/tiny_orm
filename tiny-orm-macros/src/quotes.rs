use quote::{format_ident, quote};

use crate::{
    attr::Attr,
    database::{self, DbType},
    types::{Column, PrimaryKey, ReturnObject},
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
                    ::sqlx::Result<#pk_type>
                }
            }
            ReturnType::EntireRow(return_object) => quote! {
                ::sqlx::Result<#return_object>
            },
            ReturnType::OptionalRow(return_object) => quote! {
                ::sqlx::Result<Option<#return_object>>
            },
            ReturnType::MultipleRows(return_object) => quote! {
                ::sqlx::Result<Vec<#return_object>>
            },
            ReturnType::None => quote! {
                ::sqlx::Result<()>
            },
        }
    }

    fn returning_statement(self) -> proc_macro2::TokenStream {
        // MySQL does not support the RETURNING statement
        if database::db_type() == DbType::MySQL {
            return quote! {};
        }
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
        match (database::db_type(), self) {
            (
                DbType::MySQL,
                ReturnType::PrimaryKey(Column {
                    auto_increment: true,
                    ..
                }),
            ) => quote! {
                qb.build()
                .execute(db)
                .await
                .map(|result| result.last_insert_id() as _)
            },
            (DbType::MySQL, ReturnType::PrimaryKey(primary_key)) => {
                let pk_ident = primary_key.ident;
                quote! {
                    qb.build()
                    .execute(db)
                    .await?;

                    Ok(self.#pk_ident.clone())
                }
            }
            (_, ReturnType::PrimaryKey(_)) => quote! {
                qb.build()
                .fetch_one(db)
                .await
                .map(|row| row.get(0))
            },
            (_, ReturnType::EntireRow(_)) => quote! {
                qb.build_query_as()
                .fetch_one(db)
                .await
            },
            (_, ReturnType::OptionalRow(_)) => quote! {
                qb.build_query_as()
                .fetch_optional(db)
                .await
            },
            (_, ReturnType::MultipleRows(_)) => quote! {
                qb.build_query_as()
                .fetch_all(db)
                .await
            },
            (_, ReturnType::None) => quote! {
                qb.build()
                .execute(db)
                .await
                .map(|_| ())
            },
        }
    }
}

pub fn get_table_name(attr: &Attr) -> proc_macro2::TokenStream {
    let table_name = attr.parsed_struct.table_name.0.as_str();
    quote! {
        pub fn table_name<'a>() -> &'a str {
            #table_name
        }
    }
}

pub fn get_by_id_fn(attr: &Attr) -> proc_macro2::TokenStream {
    let db_type_ident = database::db_type().to_ident();
    let return_type = ReturnType::OptionalRow(attr.clone().parsed_struct.return_object);
    let function_output = return_type.clone().function_output();
    let query_builder_execution = return_type.query_builder_execution();
    let table_name = attr.parsed_struct.table_name.clone().to_string();

    let (pk_name, pk_type) = match attr.primary_key {
        Some(ref pk) => (&pk.name, &pk._type),
        None => panic!("No primary key field found which is mandatory for the 'get' operation"),
    };

    let where_statement = match attr.soft_deletion {
        true => quote! {
            qb.push(" WHERE deleted_at IS NULL AND ");
        },
        false => quote! {
            qb.push(" WHERE ");
        },
    };
    quote! {
        pub async fn get_by_id<'e, E>(db: E, id: &#pk_type) -> #function_output
        where
            E: ::sqlx::#db_type_ident<'e>
        {
        let mut qb = ::sqlx::QueryBuilder::new("SELECT * FROM ");
            qb.push(#table_name);
            #where_statement
            qb.push(#pk_name);
            qb.push(" = ");
            qb.push_bind(id);

            #query_builder_execution
        }
    }
}

pub fn list_all_fn(attr: &Attr) -> proc_macro2::TokenStream {
    let db_type_ident = database::db_type().to_ident();
    let return_type = ReturnType::MultipleRows(attr.clone().parsed_struct.return_object);
    let function_output = return_type.clone().function_output();
    let query_builder_execution = return_type.query_builder_execution();
    let table_name = attr.parsed_struct.table_name.clone().to_string();

    let where_statement = match attr.soft_deletion {
        true => quote! {
            qb.push(" WHERE deleted_at IS NULL ");
        },
        false => quote! {},
    };

    quote! {
        pub async fn list_all<'e, E>(db: E) -> #function_output
        where
            E: ::sqlx::#db_type_ident<'e>
        {
            let mut qb = ::sqlx::QueryBuilder::new("SELECT * FROM ");
            qb.push(#table_name);
            #where_statement
            #query_builder_execution
        }
    }
}

pub fn create_fn(attr: &Attr) -> proc_macro2::TokenStream {
    let db_type = database::db_type();
    let db_type_ident = db_type.clone().to_ident();
    let table_name = attr.parsed_struct.table_name.clone().to_string();

    let mysql_specific_error = r#"MySQL does not support the `RETURNING *` statement
    Thus it's not possible to create a record without a known primary_key column with the `Table` macro.
    If an auto increment column is used, set a dummy value and it will be ignored."#;
    let return_type = match (&db_type, attr.primary_key.clone()) {
        (&DbType::MySQL, None) => panic!("{mysql_specific_error}"),
        (_, None) => ReturnType::EntireRow(attr.parsed_struct.return_object.clone()),
        (_, Some(primary_key)) => ReturnType::PrimaryKey(primary_key),
    };

    let function_output = return_type.clone().function_output();
    let returning_statement = return_type.clone().returning_statement();
    let query_builder_execution = return_type.query_builder_execution();

    let mut field_str_quote = Vec::new();
    let mut field_values_quote = Vec::new();

    for column in attr.columns.iter() {
        if column.auto_increment {
            continue;
        }
        let str_quote = if column.use_set_options() {
            let column_ident = &column.ident;
            let column_name = &column.name;
            quote! {
                if self.#column_ident.is_set() {
                    fields_str.push(#column_name);
                }
            }
        } else {
            let column_name = &column.name;
            quote! {
                fields_str.push(#column_name);
            }
        };
        field_str_quote.push(str_quote);

        let value_quote = if column.use_set_options() {
            let column_ident = &column.ident;
            quote! {
                if let SetOption::Set(v) = &self.#column_ident {
                    separated.push_bind(v);
                }
            }
        } else {
            let column_ident = &column.ident;
            quote! {
                separated.push_bind(&self.#column_ident);
            }
        };
        field_values_quote.push(value_quote);
    }

    quote! {
        pub async fn create<'e, E>(&self, db: E) -> #function_output
        where
            E: ::sqlx::#db_type_ident<'e>
        {
            let mut fields_str = Vec::new();

            #(#field_str_quote)*

            let mut qb = ::sqlx::QueryBuilder::new("INSERT INTO ");
            qb.push(#table_name);
            qb.push(" (");
            qb.push(fields_str.join(", "));
            qb.push(") VALUES (");

            let mut separated = qb.separated(", ");
            #(#field_values_quote)*
            separated.push_unseparated(")");

            #returning_statement

            #query_builder_execution
        }
    }
}

pub fn update_fn(attr: &Attr) -> proc_macro2::TokenStream {
    let db_type_ident = database::db_type().to_ident();

    let self_ident = format_ident!("Self");
    let return_type = match (database::db_type(), &attr.parsed_struct.return_object) {
        (DbType::MySQL, _) => ReturnType::None, // MySQL is not capable to return the entire row.
        (_, ident) if ident == &self_ident => ReturnType::None,
        (_, _) => ReturnType::EntireRow(attr.parsed_struct.return_object.clone()),
    };

    let function_output = return_type.clone().function_output();
    let query_builder_execution = return_type.clone().query_builder_execution();
    let returning_statement = return_type.returning_statement();

    let table_name = attr.parsed_struct.table_name.clone().to_string();
    let (pk_name, pk_ident) = match attr.primary_key {
        Some(ref pk) => (&pk.name, &pk.ident),
        None => panic!("No primary key field found"),
    };
    let mut fields_quote = Vec::new();

    for column in attr.columns.iter() {
        if column.auto_increment || column.primary_key {
            continue;
        }
        let column_ident = &column.ident;
        let column_name = &column.name;

        let quote = quote! {
            if !first {
                qb.push(", ");
            }
            qb.push(#column_name);
            qb.push(" = ");
            qb.push_bind(&self.#column_ident);
            first = false;
        };

        let str_quote = if column.use_set_options() {
            quote! {
                if self.#column_ident.is_set() {
                    #quote
                }
            }
        } else {
            quote
        };
        fields_quote.push(str_quote);
    }

    let where_statement = match attr.soft_deletion {
        true => quote! {
            qb.push(" WHERE deleted_at IS NULL AND ");
        },
        false => quote! {
            qb.push(" WHERE ");
        },
    };

    quote! {
        pub async fn update<'e, E>(&self, db: E) -> #function_output
        where
            E: ::sqlx::#db_type_ident<'e>
        {
            let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
            qb.push(#table_name);
            qb.push(" SET ");

            let mut first = true;
            #(#fields_quote)*

            #where_statement
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
    let table_name = attr.parsed_struct.table_name.clone().to_string();
    let (pk_name, pk_ident) = match attr.primary_key {
        Some(ref pk) => (&pk.name, &pk.ident),
        None => panic!("No primary key field found"),
    };
    let delete_statement = match (attr.soft_deletion, database::db_type()) {
        (true, DbType::Postgres) => quote! {
            let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
            qb.push(#table_name);
            qb.push(" SET deleted_at = NOW() ");
        },
        (true, DbType::MySQL) => quote! {
            let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
            qb.push(#table_name);
            qb.push(" SET deleted_at = CURRENT_TIMESTAMP ");
        },
        (true, DbType::Sqlite) => quote! {
            let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
            qb.push(#table_name);
            qb.push(" SET deleted_at = DATETIME('now') ");
        },
        (false, _) => quote! {
            let mut qb = ::sqlx::QueryBuilder::new("DELETE FROM ");
            qb.push(#table_name);
        },
    };
    let where_statement = match attr.soft_deletion {
        true => quote! {
            qb.push(" WHERE deleted_at IS NULL AND ");
        },
        false => quote! {
            qb.push(" WHERE ");
        },
    };
    quote! {
        pub async fn delete<'e, E>(&self, db: E) -> #function_output
        where
            E: ::sqlx::#db_type_ident<'e>
        {
            #delete_statement
            #where_statement
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
    use syn::Ident;

    pub fn clean_tokens(tokens: proc_macro2::TokenStream) -> String {
        tokens.to_string().replace([' ', '\n'], "")
    }

    #[cfg(feature = "mysql")]
    fn db_ident() -> Ident {
        format_ident!("MySqlExecutor")
    }
    #[cfg(feature = "postgres")]
    fn db_ident() -> Ident {
        format_ident!("PgExecutor")
    }
    #[cfg(feature = "sqlite")]
    fn db_ident() -> Ident {
        format_ident!("SqliteExecutor")
    }

    mod simple_attr {
        use quote::format_ident;
        use syn::parse_quote;

        use crate::types::{Column, Operation, ParsedStruct};

        use super::*;

        fn input(auto_increment: bool, soft_deletion: bool) -> Attr {
            let mut primary_key = Column::new("id", parse_quote!(i64));
            primary_key.set_primary_key();
            if auto_increment {
                primary_key.set_auto_increment();
            };
            let parsed_struct = ParsedStruct::new(
                &format_ident!("Contact"),
                Some("contact".to_string()),
                Some(format_ident!("Self")),
            );
            Attr {
                parsed_struct,
                primary_key: Some(primary_key.clone()),
                columns: vec![
                    primary_key,
                    Column::new("created_at", parse_quote!(DateTime<Utc>)),
                    Column::new("updated_at", parse_quote!(DateTime<Utc>)),
                    Column::new("last_name", parse_quote!(String)),
                ],
                operations: Operation::all(),
                soft_deletion,
            }
        }

        #[test]
        fn test_table_name() {
            let generated = clean_tokens(get_table_name(&input(false, false)));
            let expected = clean_tokens(quote! {
                pub fn table_name<'a>() -> &'a str {
                    "contact"
                }
            });
            assert_eq!(generated, expected);
        }

        #[cfg(not(feature = "mysql"))]
        #[test]
        fn test_generate_create_method() {
            let db_ident = db_ident();
            let generated = clean_tokens(create_fn(&input(false, false)));

            let expected = clean_tokens(quote! {
                pub async fn create<'e, E>(&self, db: E) -> ::sqlx::Result<i64>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut fields_str = Vec::new();
                    fields_str.push("id");
                    fields_str.push("created_at");
                    fields_str.push("updated_at");
                    fields_str.push("last_name");

                    let mut qb = ::sqlx::QueryBuilder::new("INSERT INTO ");
                    qb.push("contact");
                    qb.push(" (");
                    qb.push(fields_str.join(", "));
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

        #[cfg(feature = "mysql")]
        #[test]
        fn test_generate_create_method() {
            let generated = clean_tokens(create_fn(&input(false, false)));

            let expected = clean_tokens(quote! {
                pub async fn create<'e, E>(&self, db: E) -> ::sqlx::Result<i64>
                where
                    E: ::sqlx::MySqlExecutor<'e>
                {
                    let mut fields_str = Vec::new();
                    fields_str.push("id");
                    fields_str.push("created_at");
                    fields_str.push("updated_at");
                    fields_str.push("last_name");

                    let mut qb = ::sqlx::QueryBuilder::new("INSERT INTO ");
                    qb.push("contact");
                    qb.push(" (");
                    qb.push(fields_str.join(", "));
                    qb.push(") VALUES (");

                    let mut separated = qb.separated(", ");
                    separated.push_bind(&self.id);
                    separated.push_bind(&self.created_at);
                    separated.push_bind(&self.updated_at);
                    separated.push_bind(&self.last_name);
                    separated.push_unseparated(")");

                    qb.build()
                    .execute(db)
                    .await?;

                    Ok(self.id.clone())
                }
            });

            assert_eq!(generated, expected);
        }

        #[cfg(not(feature = "mysql"))]
        #[test]
        fn test_generate_create_method_with_auto_primary_key() {
            let db_ident = db_ident();
            let generated = clean_tokens(create_fn(&input(true, false)));

            let expected = clean_tokens(quote! {
                pub async fn create<'e, E>(&self, db: E) -> ::sqlx::Result<i64>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut fields_str = Vec::new();
                    fields_str.push("created_at");
                    fields_str.push("updated_at");
                    fields_str.push("last_name");

                    let mut qb = ::sqlx::QueryBuilder::new("INSERT INTO ");
                    qb.push("contact");
                    qb.push(" (");
                    qb.push(fields_str.join(", "));
                    qb.push(") VALUES (");

                    let mut separated = qb.separated(", ");
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

        #[cfg(feature = "mysql")]
        #[test]
        fn test_generate_create_method_with_auto_primary_key() {
            let generated = clean_tokens(create_fn(&input(true, false)));

            let expected = clean_tokens(quote! {
                pub async fn create<'e, E>(&self, db: E) -> ::sqlx::Result<i64>
                where
                    E: ::sqlx::MySqlExecutor<'e>
                {
                    let mut fields_str = Vec::new();
                    fields_str.push("created_at");
                    fields_str.push("updated_at");
                    fields_str.push("last_name");

                    let mut qb = ::sqlx::QueryBuilder::new("INSERT INTO ");
                    qb.push("contact");
                    qb.push(" (");
                    qb.push(fields_str.join(", "));
                    qb.push(") VALUES (");

                    let mut separated = qb.separated(", ");
                    separated.push_bind(&self.created_at);
                    separated.push_bind(&self.updated_at);
                    separated.push_bind(&self.last_name);
                    separated.push_unseparated(")");

                    qb.build()
                    .execute(db)
                    .await
                    .map(|result|result.last_insert_id()as_)
                }
            });

            assert_eq!(generated, expected);
        }

        #[test]
        fn test_generate_get_by_id_method() {
            let db_ident = db_ident();
            let generated = clean_tokens(get_by_id_fn(&input(false, false)));

            let expected = clean_tokens(quote! {
                pub async fn get_by_id<'e, E>(db: E, id: &i64) -> ::sqlx::Result<Option<Self>>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("SELECT * FROM ");
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
        fn test_generate_get_by_id_method_with_soft_deletion() {
            let db_ident = db_ident();
            let generated = clean_tokens(get_by_id_fn(&input(false, true)));

            let expected = clean_tokens(quote! {
                pub async fn get_by_id<'e, E>(db: E, id: &i64) -> ::sqlx::Result<Option<Self>>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("SELECT * FROM ");
                    qb.push("contact");
                    qb.push(" WHERE deleted_at IS NULL AND ");
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
            let db_ident = db_ident();
            let generated = clean_tokens(list_all_fn(&input(false, false)));

            let expected = clean_tokens(quote! {
                pub async fn list_all<'e, E>(db: E) -> ::sqlx::Result<Vec<Self>>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("SELECT * FROM ");
                    qb.push("contact");

                    qb.build_query_as()
                    .fetch_all(db)
                    .await
                }
            });

            assert_eq!(generated, expected);
        }
        #[test]
        fn test_generate_list_all_method_with_soft_deletion() {
            let db_ident = db_ident();
            let generated = clean_tokens(list_all_fn(&input(false, true)));

            let expected = clean_tokens(quote! {
                pub async fn list_all<'e, E>(db: E) -> ::sqlx::Result<Vec<Self>>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("SELECT * FROM ");
                    qb.push("contact");
                    qb.push("WHERE deleted_at IS NULL ");

                    qb.build_query_as()
                    .fetch_all(db)
                    .await
                }
            });

            assert_eq!(generated, expected);
        }

        #[test]
        fn test_generate_update_method() {
            let db_ident = db_ident();
            let generated = clean_tokens(update_fn(&input(false, false)));

            let expected = clean_tokens(quote! {
                pub async fn update<'e, E>(&self, db: E) -> ::sqlx::Result<()>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
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

                    qb.build()
                    .execute(db)
                    .await
                    .map(|_|())
                }
            });

            assert_eq!(generated, expected);
        }
        #[test]
        fn test_generate_update_method_with_soft_deletion() {
            let db_ident = db_ident();
            let generated = clean_tokens(update_fn(&input(false, true)));

            let expected = clean_tokens(quote! {
                pub async fn update<'e, E>(&self, db: E) -> ::sqlx::Result<()>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
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

                    qb.push(" WHERE deleted_at IS NULL AND ");
                    qb.push("id");
                    qb.push(" = ");
                    qb.push_bind(&self.id);

                    qb.build()
                    .execute(db)
                    .await
                    .map(|_|())
                }
            });

            assert_eq!(generated, expected);
        }

        #[test]
        fn test_generate_delete_method() {
            let db_ident = db_ident();
            let generated = clean_tokens(delete_fn(&input(false, false)));

            let expected = clean_tokens(quote! {
                pub async fn delete<'e, E>(&self, db: E) -> ::sqlx::Result<()>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("DELETE FROM ");
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
        #[cfg(feature = "sqlite")]
        #[test]
        fn test_generate_delete_method_with_soft_deletion() {
            let db_ident = db_ident();
            let generated = clean_tokens(delete_fn(&input(false, true)));

            let expected = clean_tokens(quote! {
                pub async fn delete<'e, E>(&self, db: E) -> ::sqlx::Result<()>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
                    qb.push("contact");
                    qb.push(" SET deleted_at = DATETIME('now') ");
                    qb.push(" WHERE deleted_at IS NULL AND ");
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
        #[cfg(feature = "postgres")]
        #[test]
        fn test_generate_delete_method_with_soft_deletion() {
            let db_ident = db_ident();
            let generated = clean_tokens(delete_fn(&input(false, true)));

            let expected = clean_tokens(quote! {
                pub async fn delete<'e, E>(&self, db: E) -> ::sqlx::Result<()>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
                    qb.push("contact");
                    qb.push(" SET deleted_at = NOW() ");
                    qb.push(" WHERE deleted_at IS NULL AND ");
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
        #[cfg(feature = "mysql")]
        #[test]
        fn test_generate_delete_method_with_soft_deletion() {
            let db_ident = db_ident();
            let generated = clean_tokens(delete_fn(&input(false, true)));

            let expected = clean_tokens(quote! {
                pub async fn delete<'e, E>(&self, db: E) -> ::sqlx::Result<()>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
                    qb.push("contact");
                    qb.push(" SET deleted_at = CURRENT_TIMESTAMP ");
                    qb.push(" WHERE deleted_at IS NULL AND ");
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
        use super::*;
        use crate::attr::Attr;
        use crate::types::*;
        use syn::parse_quote;

        #[cfg(not(feature = "mysql"))]
        #[test]
        fn test_custom_output_create() {
            let db_ident = db_ident();
            let parsed_struct = ParsedStruct::new(&format_ident!("NewContact"), None, None);
            let input = Attr {
                parsed_struct,
                primary_key: None,
                columns: vec![
                    Column::new("first_name", parse_quote!(String)),
                    Column::new("last_name", parse_quote!(String)),
                    Column::new("email", parse_quote!(String)),
                ],
                operations: vec![Operation::Create],
                soft_deletion: false,
            };

            let generated = clean_tokens(create_fn(&input));

            let expected = clean_tokens(quote! {
                pub async fn create<'e, E>(&self, db: E) -> ::sqlx::Result<Contact>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut fields_str = Vec::new();
                    fields_str.push("first_name");
                    fields_str.push("last_name");
                    fields_str.push("email");

                    let mut qb = ::sqlx::QueryBuilder::new("INSERT INTO ");
                    qb.push("contact");
                    qb.push(" (");
                    qb.push(fields_str.join(", "));
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

        #[cfg(feature = "mysql")]
        #[test]
        #[should_panic]
        fn test_custom_output_create() {
            let parsed_struct = ParsedStruct::new(&format_ident!("NewContact"), None, None);
            let input = Attr {
                parsed_struct,
                primary_key: None,
                columns: vec![
                    Column::new("first_name", parse_quote!(String)),
                    Column::new("last_name", parse_quote!(String)),
                    Column::new("email", parse_quote!(String)),
                ],
                operations: vec![Operation::Create],
                soft_deletion: false,
            };

            create_fn(&input);
        }

        #[cfg(not(feature = "mysql"))]
        #[test]
        fn test_setoption_create_null_pk() {
            let db_ident = db_ident();
            let parsed_struct = ParsedStruct::new(&format_ident!("NewContact"), None, None);
            let input = Attr {
                parsed_struct,
                primary_key: None,
                columns: vec![
                    Column::new("first_name", parse_quote!(SetOption<String>)),
                    Column::new("last_name", parse_quote!(SetOption<String>)),
                    Column::new("email", parse_quote!(String)),
                ],
                operations: vec![Operation::Create],
                soft_deletion: false,
            };

            let generated = clean_tokens(create_fn(&input));

            let expected = clean_tokens(quote! {
                pub async fn create<'e, E>(&self, db: E) -> ::sqlx::Result<Contact>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut fields_str = Vec::new();
                    if self.first_name.is_set() {
                        fields_str.push("first_name");
                    }
                    if self.last_name.is_set() {
                        fields_str.push("last_name");
                    }
                    fields_str.push("email");

                    let mut qb = ::sqlx::QueryBuilder::new("INSERT INTO ");
                    qb.push("contact");
                    qb.push(" (");
                    qb.push(fields_str.join(", "));
                    qb.push(") VALUES (");

                    let mut separated = qb.separated(", ");
                    if let SetOption::Set(v) = &self.first_name {
                        separated.push_bind(v);
                    }
                    if let SetOption::Set(v) = &self.last_name {
                        separated.push_bind(v);
                    }
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

        #[cfg(not(feature = "mysql"))]
        #[test]
        fn test_setoption_create_auto_generated_pk() {
            let db_ident = db_ident();
            let parsed_struct = ParsedStruct::new(&format_ident!("NewContact"), None, None);
            let mut primary_key = Column::new("incremental_id", parse_quote!(i64));
            primary_key.set_primary_key();
            primary_key.set_auto_increment();
            let input = Attr {
                parsed_struct,
                primary_key: Some(primary_key.clone()),
                columns: vec![
                    primary_key,
                    Column::new("first_name", parse_quote!(SetOption<String>)),
                    Column::new("last_name", parse_quote!(SetOption<String>)),
                    Column::new("email", parse_quote!(String)),
                ],
                operations: vec![Operation::Create],
                soft_deletion: false,
            };

            let generated = clean_tokens(create_fn(&input));

            let expected = clean_tokens(quote! {
                pub async fn create<'e, E>(&self, db: E) -> ::sqlx::Result<i64>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut fields_str = Vec::new();
                    if self.first_name.is_set() {
                        fields_str.push("first_name");
                    }
                    if self.last_name.is_set() {
                        fields_str.push("last_name");
                    }
                    fields_str.push("email");

                    let mut qb = ::sqlx::QueryBuilder::new("INSERT INTO ");
                    qb.push("contact");
                    qb.push(" (");
                    qb.push(fields_str.join(", "));
                    qb.push(") VALUES (");

                    let mut separated = qb.separated(", ");
                    if let SetOption::Set(v) = &self.first_name {
                        separated.push_bind(v);
                    }
                    if let SetOption::Set(v) = &self.last_name {
                        separated.push_bind(v);
                    }
                    separated.push_bind(&self.email);
                    separated.push_unseparated(")");

                    qb.push("RETURNING");
                    qb.push("incremental_id");

                    qb.build()
                    .fetch_one(db)
                    .await
                    .map(|row|row.get(0))
                }
            });
            assert_eq!(generated, expected);
        }

        #[cfg(not(feature = "mysql"))]
        #[test]
        fn test_setoption_create_custom_pk() {
            let db_ident = db_ident();
            let parsed_struct = ParsedStruct::new(&format_ident!("NewContact"), None, None);
            let mut primary_key = Column::new("uuid", parse_quote!(Uuid));
            primary_key.set_primary_key();
            let input = Attr {
                parsed_struct,
                primary_key: Some(primary_key.clone()),
                columns: vec![
                    primary_key,
                    Column::new("first_name", parse_quote!(SetOption<String>)),
                    Column::new("last_name", parse_quote!(SetOption<String>)),
                    Column::new("email", parse_quote!(String)),
                ],
                operations: vec![Operation::Create],
                soft_deletion: false,
            };

            let generated = clean_tokens(create_fn(&input));

            let expected = clean_tokens(quote! {
                pub async fn create<'e, E>(&self, db: E) -> ::sqlx::Result<Uuid>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut fields_str = Vec::new();
                    fields_str.push("uuid");
                    if self.first_name.is_set() {
                        fields_str.push("first_name");
                    }
                    if self.last_name.is_set() {
                        fields_str.push("last_name");
                    }
                    fields_str.push("email");

                    let mut qb = ::sqlx::QueryBuilder::new("INSERT INTO ");
                    qb.push("contact");
                    qb.push(" (");
                    qb.push(fields_str.join(", "));
                    qb.push(") VALUES (");

                    let mut separated = qb.separated(", ");
                    separated.push_bind(&self.uuid);
                    if let SetOption::Set(v) = &self.first_name {
                        separated.push_bind(v);
                    }
                    if let SetOption::Set(v) = &self.last_name {
                        separated.push_bind(v);
                    }
                    separated.push_bind(&self.email);
                    separated.push_unseparated(")");

                    qb.push("RETURNING");
                    qb.push("uuid");

                    qb.build()
                    .fetch_one(db)
                    .await
                    .map(|row|row.get(0))
                }
            });
            assert_eq!(generated, expected);
        }

        #[cfg(not(feature = "mysql"))]
        #[test]
        fn test_custom_output_update() {
            let db_ident = db_ident();
            let parsed_struct = ParsedStruct::new(&format_ident!("UpdateContact"), None, None);
            let mut primary_key = Column::new("custom_id", parse_quote!(i64));
            primary_key.set_primary_key();

            let input = Attr {
                parsed_struct,
                primary_key: Some(primary_key.clone()),
                columns: vec![primary_key, Column::new("first_name", parse_quote!(String))],
                operations: vec![Operation::Update],
                soft_deletion: false,
            };

            let generated = clean_tokens(update_fn(&input));

            let expected = clean_tokens(quote! {
                pub async fn update<'e, E>(&self, db: E) -> ::sqlx::Result<Contact>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
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

        #[cfg(feature = "mysql")]
        #[test]
        fn test_custom_output_update() {
            let parsed_struct = ParsedStruct::new(&format_ident!("UpdateContact"), None, None);
            let mut primary_key = Column::new("custom_id", parse_quote!(i64));
            primary_key.set_primary_key();

            let input = Attr {
                parsed_struct,
                primary_key: Some(primary_key.clone()),
                columns: vec![primary_key, Column::new("first_name", parse_quote!(String))],
                operations: vec![Operation::Update],
                soft_deletion: false,
            };

            let generated = clean_tokens(update_fn(&input));

            let expected = clean_tokens(quote! {
                pub async fn update<'e, E>(&self, db: E) -> ::sqlx::Result<()>
                where
                    E: ::sqlx::MySqlExecutor<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
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

                    qb.build()
                    .execute(db)
                    .await
                    .map(|_|())
                }
            });

            assert_eq!(generated, expected);
        }

        #[cfg(not(feature = "mysql"))]
        #[test]
        fn test_custom_output_update_setoption() {
            let db_ident = db_ident();
            let parsed_struct = ParsedStruct::new(&format_ident!("UpdateContact"), None, None);
            let mut primary_key = Column::new("custom_id", parse_quote!(i64));
            primary_key.set_primary_key();

            let input = Attr {
                parsed_struct,
                primary_key: Some(primary_key.clone()),
                columns: vec![
                    primary_key,
                    Column::new("first_name", parse_quote!(SetOption<String>)),
                    Column::new("last_name", parse_quote!(String)),
                ],
                operations: vec![Operation::Update],
                soft_deletion: false,
            };

            let generated = clean_tokens(update_fn(&input));

            let expected = clean_tokens(quote! {
                pub async fn update<'e, E>(&self, db: E) -> ::sqlx::Result<Contact>
                where
                    E: ::sqlx::#db_ident<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
                    qb.push("contact");
                    qb.push(" SET ");

                    let mut first = true;

                    if self.first_name.is_set() {
                        if !first {
                            qb.push(",");
                        }
                        qb.push("first_name");
                        qb.push(" = ");
                        qb.push_bind(&self.first_name);
                        first = false;
                    }
                    if !first {
                        qb.push(",");
                    }
                    qb.push("last_name");
                    qb.push(" = ");
                    qb.push_bind(&self.last_name);
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

        #[cfg(feature = "mysql")]
        #[test]
        fn test_custom_output_update_setoption() {
            let parsed_struct = ParsedStruct::new(&format_ident!("UpdateContact"), None, None);
            let mut primary_key = Column::new("custom_id", parse_quote!(i64));
            primary_key.set_primary_key();

            let input = Attr {
                parsed_struct,
                primary_key: Some(primary_key.clone()),
                columns: vec![
                    primary_key,
                    Column::new("first_name", parse_quote!(SetOption<String>)),
                    Column::new("last_name", parse_quote!(String)),
                ],
                operations: vec![Operation::Update],
                soft_deletion: false,
            };

            let generated = clean_tokens(update_fn(&input));

            let expected = clean_tokens(quote! {
                pub async fn update<'e, E>(&self, db: E) -> ::sqlx::Result<()>
                where
                    E: ::sqlx::MySqlExecutor<'e>
                {
                    let mut qb = ::sqlx::QueryBuilder::new("UPDATE ");
                    qb.push("contact");
                    qb.push(" SET ");

                    let mut first = true;

                    if self.first_name.is_set() {
                        if !first {
                            qb.push(",");
                        }
                        qb.push("first_name");
                        qb.push(" = ");
                        qb.push_bind(&self.first_name);
                        first = false;
                    }
                    if !first {
                        qb.push(",");
                    }
                    qb.push("last_name");
                    qb.push(" = ");
                    qb.push_bind(&self.last_name);
                    first = false;

                    qb.push(" WHERE ");
                    qb.push("custom_id");
                    qb.push(" = ");
                    qb.push_bind(&self.custom_id);

                    qb.build()
                    .execute(db)
                    .await
                    .map(|_|())
                }
            });

            assert_eq!(generated, expected);
        }
    }
}
