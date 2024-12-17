use std::str::FromStr;
use syn::{
    parenthesized, parse_str, punctuated::Punctuated, token::Paren, Attribute, Data, DeriveInput,
    Expr, ExprLit, Fields, Ident, Lit, Meta, Token,
};

use crate::types::{Column, FieldNames, Operation, Operations, ParsedStruct, PrimaryKey};

const NAME_MACRO_OPERATION_ARG: &str = "tiny_orm";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    pub parsed_struct: ParsedStruct,
    pub primary_key: Option<PrimaryKey>,
    pub field_names: FieldNames,
    pub operations: Operations,
}

impl Attr {
    pub fn parse(input: DeriveInput) -> Self {
        let struct_name = input.ident;
        let (parsed_struct, operations) =
            Parser::parse_struct_macro_arguments(&struct_name, &input.attrs);
        let (primary_key, field_names) = Parser::parse_fields_macro_arguments(input.data);

        Attr {
            parsed_struct,
            primary_key,
            field_names,
            operations,
        }
    }
}

struct Parser();

impl Parser {
    fn parse_struct_macro_arguments(
        struct_name: &Ident,
        attrs: &[Attribute],
    ) -> (ParsedStruct, Operations) {
        let mut only: Option<Vec<Operation>> = None;
        let mut exclude: Option<Vec<Operation>> = None;
        let mut return_object: Option<Ident> = None;
        let mut table_name: Option<String> = None;

        for attr in attrs {
            if attr.path().is_ident(NAME_MACRO_OPERATION_ARG) {
                let nested = attr
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    .unwrap();
                for meta in nested {
                    match meta {
                        Meta::NameValue(name_value) if name_value.path.is_ident("table_name") => {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(lit_str),
                                ..
                            }) = name_value.clone().value
                            {
                                table_name = Some(lit_str.value().clone());
                            };
                        }
                        Meta::NameValue(name_value)
                            if name_value.path.is_ident("return_object") =>
                        {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(lit_str),
                                ..
                            }) = name_value.clone().value
                            {
                                return_object = Some(
                                    parse_str::<Ident>(&lit_str.value())
                                        .expect("Failed to parse return_object as identifier"),
                                );
                            };
                        }
                        Meta::NameValue(name_value) if name_value.path.is_ident("only") => {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(lit_str),
                                ..
                            }) = name_value.clone().value
                            {
                                if only.is_some() {
                                    panic!("The 'only' keyword has already been specified")
                                };
                                only = Some(
                                    lit_str
                                        .value()
                                        .split(',')
                                        .map(|s| {
                                            Operation::from_str(s.trim()).unwrap_or_else(|_| {
                                                panic!("Operation {s} is not supported")
                                            })
                                        })
                                        .collect(),
                                );
                            };
                        }
                        Meta::NameValue(name_value) if name_value.path.is_ident("exclude") => {
                            if let Expr::Lit(ExprLit {
                                lit: Lit::Str(lit_str),
                                ..
                            }) = name_value.clone().value
                            {
                                if exclude.is_some() {
                                    panic!("The 'exclude' keyword has already been specified")
                                };
                                exclude = Some(
                                    lit_str
                                        .value()
                                        .split(',')
                                        .map(|s| {
                                            Operation::from_str(s.trim()).unwrap_or_else(|_| {
                                                panic!("Operation {s} is not supported")
                                            })
                                        })
                                        .collect(),
                                );
                            };
                        }
                        Meta::Path(path) if path.is_ident("all") => {
                            only = Some(Operation::all());
                        }
                        _ => {
                            panic!("Error - Skip unknown name value");
                        }
                    }
                }
            }
        }

        let parsed_struct = ParsedStruct::new(struct_name, table_name, return_object);
        let operations =
            Parser::get_operations(only, exclude, parsed_struct.struct_type.default_operation())
                .expect("Cannot parse the only/exclude operations properly.");

        (parsed_struct, operations)
    }

    fn parse_fields_macro_arguments(data: Data) -> (Option<PrimaryKey>, FieldNames) {
        let mut primary_key: Option<PrimaryKey> = None;
        let mut field_names = Vec::new();

        match data {
            Data::Struct(data_struct) => match &data_struct.fields {
                Fields::Named(fields) => {
                    for field in fields.named.iter() {
                        let field_name = field
                            .ident
                            .as_ref()
                            .expect("Field ident to be something")
                            .to_string();

                        for attr in &field.attrs {
                            if attr.path().is_ident(NAME_MACRO_OPERATION_ARG) {
                                attr.parse_nested_meta(|meta| {
                                    if meta.path.is_ident("primary_key") {
                                        let mut column = Column::new(&field_name, field.ty.clone());

                                        if meta.input.peek(Paren) {
                                            let content;
                                            parenthesized!(content in meta.input);

                                            let ident: Option<Ident> = content.parse().ok();
                                            if let Some(ident) = ident {
                                                if ident == "auto" {
                                                    column.set_auto_increment();
                                                }
                                            }
                                        }
                                        primary_key = Some(column);
                                    }
                                    Ok(())
                                })
                                .unwrap_or(());
                            }
                        }

                        // Default fallbacks
                        if field_name == "id" && primary_key.is_none() {
                            primary_key = Some(Column::new(&field_name, field.ty.clone()));
                        }

                        field_names.push(field_name);
                    }
                }
                _ => panic!("Only named fields are supported"),
            },
            _ => panic!("Only structs are supported"),
        };
        (primary_key, field_names)
    }

    fn get_operations(
        only: Option<Operations>,
        exclude: Option<Operations>,
        default: Operations,
    ) -> Result<Operations, &'static str> {
        match (only, exclude) {
            (None, None) => Ok(default),
            (Some(_), Some(_)) => Err("Only and Exclude are specified which is not allowed"),
            (Some(values), None) => Ok(values),
            (None, Some(values)) => Ok(Operation::all()
                .into_iter()
                .filter(|op| !values.contains(op))
                .collect()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod generic {
        use syn::parse_quote;

        use super::Column;

        #[test]
        fn test_set_auto_increment() {
            let mut column = Column::new("col_name", parse_quote!(i32));
            assert!(!column.auto_increment);
            column.set_auto_increment();
            assert!(column.auto_increment);
        }
    }

    mod operation_tests {
        use super::{Operation, Parser};

        // fn struct_name() {
        //     TableName
        // }

        #[test]
        fn test_get_them_all_by_default() {
            let mut result = Parser::get_operations(None, None, Operation::all()).unwrap();
            result.sort();
            assert_eq!(
                result,
                vec![
                    Operation::Get,
                    Operation::List,
                    Operation::Create,
                    Operation::Update,
                    Operation::Delete
                ]
            );
        }

        #[test]
        fn test_get_default_by_default() {
            let mut result =
                Parser::get_operations(None, None, vec![Operation::Create, Operation::Delete])
                    .unwrap();
            result.sort();
            assert_eq!(result, vec![Operation::Create, Operation::Delete]);
        }

        #[test]
        fn test_only_and_exclude() {
            let only = Some(vec![Operation::Create]);
            let exclude = Some(vec![Operation::Delete]);
            let result = Parser::get_operations(only, exclude, Operation::all());
            assert!(result.is_err());
        }

        #[test]
        fn test_only_one_value() {
            let only = vec![Operation::Create];
            let result = Parser::get_operations(Some(only.clone()), None, Operation::all());
            assert_eq!(result.unwrap(), only);
        }

        #[test]
        fn test_only_multiple_values() {
            let only = vec![Operation::Update, Operation::Delete];
            let result = Parser::get_operations(Some(only.clone()), None, Operation::all());
            assert_eq!(result.unwrap(), only);
        }

        #[test]
        fn test_exclude_one_value() {
            let exclude = Some(vec![Operation::Create]);
            let mut result = Parser::get_operations(None, exclude, Operation::all()).unwrap();
            result.sort();
            assert_eq!(
                result,
                vec![
                    Operation::Get,
                    Operation::List,
                    Operation::Update,
                    Operation::Delete
                ]
            );
        }

        #[test]
        fn test_exclude_multiple_values() {
            let exclude = Some(vec![Operation::Update, Operation::Delete]);
            let mut result = Parser::get_operations(None, exclude, Operation::all()).unwrap();
            result.sort();
            assert_eq!(
                result,
                vec![Operation::Get, Operation::List, Operation::Create]
            );
        }
    }

    mod parse_struct_macro_arguments {
        use quote::format_ident;
        use syn::parse_quote;

        use crate::attr::Parser;
        use crate::types::{Operation, StructType};

        #[test]
        fn test_parse_only_attribute_alone() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![parse_quote!(#[tiny_orm(only = "create,get")])];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(parsed_struct.name.to_string(), "MyStruct".to_string());
            assert_eq!(parsed_struct.struct_type, StructType::Generic);
            assert_eq!(
                parsed_struct.table_name.0.to_string(),
                "my_struct".to_string()
            );
            assert_eq!(parsed_struct.return_object, format_ident!("Self"));
            assert_eq!(operations, vec![Operation::Create, Operation::Get]);
        }

        #[test]
        fn test_parse_exclude_attribute_alone() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![parse_quote!(#[tiny_orm(exclude = "create,get")])];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);

            assert_eq!(parsed_struct.name.to_string(), "MyStruct".to_string());
            assert_eq!(parsed_struct.struct_type, StructType::Generic);
            assert_eq!(
                parsed_struct.table_name.0.to_string(),
                "my_struct".to_string()
            );
            assert_eq!(parsed_struct.return_object, format_ident!("Self"));
            assert_eq!(
                operations,
                vec![Operation::List, Operation::Update, Operation::Delete]
            );
        }

        #[test]
        fn test_parse_table_name_attribute_alone() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![parse_quote!(#[tiny_orm(table_name = "custom_name")])];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(parsed_struct.name.to_string(), "MyStruct".to_string());
            assert_eq!(parsed_struct.struct_type, StructType::Generic);
            assert_eq!(
                parsed_struct.table_name.0.to_string(),
                "custom_name".to_string()
            );
            assert_eq!(parsed_struct.return_object, format_ident!("Self"));
            assert_eq!(
                operations,
                vec![Operation::Get, Operation::List, Operation::Delete]
            );
        }

        #[test]
        fn test_parse_return_object_attribute_alone() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![parse_quote!(#[tiny_orm(return_object = "Operation")])];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(
                parsed_struct.table_name.0.to_string(),
                "my_struct".to_string()
            );
            assert_eq!(parsed_struct.struct_type, StructType::Generic);
            assert_eq!(parsed_struct.return_object, format_ident!("Operation"));
            assert_eq!(
                operations,
                vec![Operation::Get, Operation::List, Operation::Delete]
            );
        }

        #[test]
        fn test_parse_multiple_attributes() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![
                parse_quote!(#[tiny_orm(table_name = "custom", return_object = "Operation", only = "create")]),
            ];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(parsed_struct.table_name.0.to_string(), "custom".to_string());
            assert_eq!(parsed_struct.return_object, format_ident!("Operation"));
            assert_eq!(operations, vec![Operation::Create]);
        }

        #[test]
        fn test_parse_multiple_attributes_withespace() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![
                parse_quote!(#[tiny_orm(table_name = "   custom ", return_object = "   Operation  ", only = "  create   ,  delete")]),
            ];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(parsed_struct.table_name.0.to_string(), "custom".to_string());
            assert_eq!(parsed_struct.return_object, format_ident!("Operation"));
            assert_eq!(operations, vec![Operation::Create, Operation::Delete]);
        }

        #[test]
        fn test_default_parse_create_type_of_struct() {
            let struct_name = format_ident!("NewMyStruct");
            let attrs = vec![parse_quote!(#[tiny_orm()])];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(parsed_struct.name.to_string(), "NewMyStruct".to_string());
            assert_eq!(
                parsed_struct.table_name.0.to_string(),
                "my_struct".to_string()
            );
            assert_eq!(parsed_struct.struct_type, StructType::Create);
            assert_eq!(parsed_struct.return_object, format_ident!("MyStruct"));
            assert_eq!(operations, vec![Operation::Create]);
        }

        #[test]
        fn test_default_parse_update_type_of_struct() {
            let struct_name = format_ident!("UpdateMyStruct");
            let attrs = vec![parse_quote!(#[tiny_orm()])];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(parsed_struct.name.to_string(), "UpdateMyStruct".to_string());
            assert_eq!(
                parsed_struct.table_name.0.to_string(),
                "my_struct".to_string()
            );
            assert_eq!(parsed_struct.struct_type, StructType::Update);
            assert_eq!(parsed_struct.return_object, format_ident!("MyStruct"));
            assert_eq!(operations, vec![Operation::Update]);
        }

        #[test]
        fn test_parse_create_type_of_struct_with_override() {
            let struct_name = format_ident!("NewMyStruct");
            let attrs = vec![
                parse_quote!(#[tiny_orm(table_name = "   custom ", return_object = "   Operation  ", only = "  create   ,  delete")]),
            ];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(parsed_struct.name.to_string(), "NewMyStruct".to_string());
            assert_eq!(parsed_struct.table_name.0.to_string(), "custom".to_string());
            assert_eq!(parsed_struct.struct_type, StructType::Create);
            assert_eq!(parsed_struct.return_object, format_ident!("Operation"));
            assert_eq!(operations, vec![Operation::Create, Operation::Delete]);
        }

        #[test]
        fn test_parse_update_type_of_struct_with_override() {
            let struct_name = format_ident!("UpdateMyStruct");
            let attrs = vec![
                parse_quote!(#[tiny_orm(table_name = "   custom ", return_object = "   Operation  ", only = "  create   ,  delete")]),
            ];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(parsed_struct.name.to_string(), "UpdateMyStruct".to_string());
            assert_eq!(parsed_struct.table_name.0.to_string(), "custom".to_string());
            assert_eq!(parsed_struct.struct_type, StructType::Update);
            assert_eq!(parsed_struct.return_object, format_ident!("Operation"));
            assert_eq!(operations, vec![Operation::Create, Operation::Delete]);
        }

        #[test]
        fn test_return_all_operations_for_generic_struct() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![parse_quote!(#[tiny_orm(all, table_name = "custom")])];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(parsed_struct.name.to_string(), "MyStruct".to_string());
            assert_eq!(parsed_struct.table_name.0.to_string(), "custom".to_string());
            assert_eq!(operations, Operation::all());
        }

        #[test]
        fn test_return_all_operations_for_update_struct() {
            let struct_name = format_ident!("NewMyStruct");
            let attrs = vec![parse_quote!(#[tiny_orm(all)])];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(parsed_struct.struct_type, StructType::Create);
            assert_eq!(operations, Operation::all());
        }

        #[test]
        fn test_return_all_operations_for_create_struct() {
            let struct_name = format_ident!("UpdateMyStruct");
            let attrs = vec![parse_quote!(#[tiny_orm(all)])];
            let (parsed_struct, operations) =
                Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(parsed_struct.struct_type, StructType::Update);
            assert_eq!(operations, Operation::all());
        }

        #[test]
        #[should_panic]
        fn test_cannot_pass_all_with_only() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![parse_quote!(#[tiny_orm(all, only = "create")])];
            let _ = Parser::parse_struct_macro_arguments(&struct_name, &attrs);
        }

        #[test]
        #[should_panic]
        fn test_cannot_pass_all_with_exclude() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![parse_quote!(#[tiny_orm(all, exclude = "delete")])];
            let _ = Parser::parse_struct_macro_arguments(&struct_name, &attrs);
        }
    }

    mod parse_fields_macro_arguments {
        use syn::{parse_quote, DeriveInput};

        use crate::attr::{Column, Parser};

        #[test]
        fn test_parse_default() {
            let input: DeriveInput = parse_quote! {
                struct Contact {
                    id: i64,
                    created_at: DateTime<Utc>,
                    updated_at: DateTime<Utc>,
                    last_name: String,
                }
            };

            let (primary_key, field_names) = Parser::parse_fields_macro_arguments(input.data);
            assert_eq!(primary_key, Some(Column::new("id", parse_quote!(i64))));
            assert_eq!(
                field_names,
                vec![
                    "id".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                    "last_name".to_string()
                ]
            );
        }

        #[test]
        fn test_parse_no_default_columns() {
            let input: DeriveInput = parse_quote! {
                struct Contact {
                    last_name: String,
                }
            };

            let (primary_key, field_names) = Parser::parse_fields_macro_arguments(input.data);
            assert_eq!(primary_key, None);
            assert_eq!(field_names, vec!["last_name".to_string()]);
        }

        #[test]
        fn test_parse_override_default_values() {
            let input: DeriveInput = parse_quote! {
                struct Contact {
                    #[tiny_orm(primary_key)]
                    custom_key: u32,
                    inserted_at: DateTime<Utc>,
                    something_at: DateTime<Utc>,
                    last_name: String,
                }
            };

            let (primary_key, field_names) = Parser::parse_fields_macro_arguments(input.data);
            assert_eq!(
                primary_key,
                Some(Column::new("custom_key", parse_quote!(u32)))
            );
            assert_eq!(
                field_names,
                vec![
                    "custom_key".to_string(),
                    "inserted_at".to_string(),
                    "something_at".to_string(),
                    "last_name".to_string()
                ]
            );
        }

        #[test]
        fn test_parse_override_default_values_set_auto() {
            let input: DeriveInput = parse_quote! {
                struct Contact {
                    #[tiny_orm(primary_key(auto))]
                    custom_key: u32,
                    inserted_at: DateTime<Utc>,
                    something_at: DateTime<Utc>,
                    last_name: String,
                }
            };

            let (primary_key, _) = Parser::parse_fields_macro_arguments(input.data);
            let mut pk = Column::new("custom_key", parse_quote!(u32));
            pk.set_auto_increment();
            assert_eq!(primary_key, Some(pk));
        }

        #[test]
        fn test_parse_keep_override_even_when_default_values_present() {
            let input: DeriveInput = parse_quote! {
                struct Contact {
                    #[tiny_orm(primary_key)]
                    custom_key: u32,
                    inserted_at: DateTime<Utc>,
                    something_at: DateTime<Utc>,
                    id: u32,
                    created_at: DateTime<Utc>,
                    updated_at: DateTime<Utc>,
                    last_name: String,
                }
            };

            let (primary_key, field_names) = Parser::parse_fields_macro_arguments(input.data);
            assert_eq!(
                primary_key,
                Some(Column::new("custom_key", parse_quote!(u32)))
            );
            assert_eq!(
                field_names,
                vec![
                    "custom_key".to_string(),
                    "inserted_at".to_string(),
                    "something_at".to_string(),
                    "id".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                    "last_name".to_string()
                ]
            );
        }

        #[test]
        fn test_parse_keep_override_when_default_present_and_auto_is_set() {
            let input: DeriveInput = parse_quote! {
                struct Contact {
                    #[tiny_orm(primary_key(auto))]
                    custom_key: u32,
                    inserted_at: DateTime<Utc>,
                    something_at: DateTime<Utc>,
                    id: u32,
                    created_at: DateTime<Utc>,
                    updated_at: DateTime<Utc>,
                    last_name: String,
                }
            };

            let (primary_key, _) = Parser::parse_fields_macro_arguments(input.data);
            let mut pk = Column::new("custom_key", parse_quote!(u32));
            pk.set_auto_increment();
            assert_eq!(primary_key, Some(pk));
        }
    }

    mod parse {
        use quote::format_ident;
        use syn::{parse_quote, DeriveInput};

        use crate::attr::{Column, Operation, ParsedStruct};

        use super::Attr;

        #[test]
        fn test_parse_basic_struct() {
            let input: DeriveInput = parse_quote! {
                struct Contact {
                    id: i64,
                    created_at: DateTime<Utc>,
                    updated_at: DateTime<Utc>,
                    last_name: String,
                }
            };

            let result = Attr::parse(input);
            let parsed_struct = ParsedStruct::new(&format_ident!("Contact"), None, None);
            assert_eq!(
                result,
                Attr {
                    parsed_struct,
                    primary_key: Some(Column::new("id", parse_quote!(i64))),
                    field_names: vec![
                        "id".to_string(),
                        "created_at".to_string(),
                        "updated_at".to_string(),
                        "last_name".to_string()
                    ],
                    operations: vec![Operation::Get, Operation::List, Operation::Delete],
                }
            );
        }

        #[test]
        fn test_parse_all_options_specified() {
            let input: DeriveInput = parse_quote! {
                #[tiny_orm(table_name = "specific_table", only = "create", return_object = "AnotherObject")]
                struct Contact {
                    #[tiny_orm(primary_key)]
                    custom_pk: i64,
                    custom_created_at: DateTime<Utc>,
                    custom_updated_at: DateTime<Utc>,
                    last_name: String,
                }
            };

            let result = Attr::parse(input);
            let parsed_struct = ParsedStruct::new(
                &format_ident!("Contact"),
                Some("specific_table".to_string()),
                Some(format_ident!("AnotherObject")),
            );
            assert_eq!(
                result,
                Attr {
                    parsed_struct,
                    primary_key: Some(Column::new("custom_pk", parse_quote!(i64))),
                    field_names: vec![
                        "custom_pk".to_string(),
                        "custom_created_at".to_string(),
                        "custom_updated_at".to_string(),
                        "last_name".to_string()
                    ],
                    operations: vec![Operation::Create],
                }
            );
        }
    }
}
