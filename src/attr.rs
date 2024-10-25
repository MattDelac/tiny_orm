use std::{fmt, str::FromStr};

use proc_macro2::Span;
use quote::format_ident;
use convert_case::{Case, Casing};
use syn::{parse_str, punctuated::Punctuated, Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, Meta, Token, Type};

const NAME_MACRO_OPERATION_ARG: &str = "tiny_orm";
const ALL_OPERATIONS: [&str; 5] = ["get", "list", "create", "update", "delete"];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operation {
    Get,
    List,
    Create,
    Update,
    Delete,
}

impl Operation {
    pub fn all() -> Vec<Operation> {
        vec![Operation::Get, Operation::List, Operation::Create, Operation::Update, Operation::Delete]
    }
}

impl FromStr for Operation {
    type Err = ();

    fn from_str(input: &str) -> Result<Operation, Self::Err> {
        match input {
            "get"  => Ok(Operation::Get),
            "list"  => Ok(Operation::List),
            "create"  => Ok(Operation::Create),
            "update" => Ok(Operation::Update),
            "delete" => Ok(Operation::Delete),
            _      => Err(()),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub ident: Ident,
    pub _type: Type,
}
impl Column {
    pub fn new(name: String, _type: Type) -> Self {
        Self {
            name: name.clone(),
            ident: format_ident!("{}", name),
            _type
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableName(pub String);
impl TableName {
    pub fn new(input: String) -> Self {
        Self(input.to_case(Case::Snake))
    }

    pub fn to_ident(&self) -> Ident {
        Ident::new(&self.0, Span::call_site())
    }
}

impl fmt::Display for TableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type StructName = Ident;
pub type PrimaryKey = Column;
pub type CreatedAt = Column;
pub type UpdatedAt = Column;
pub type ReturnObject = Ident;
pub type FieldNames = Vec<String>;
pub type Operations = Vec<Operation>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    pub struct_name: StructName,
    pub table_name: TableName,
    pub return_object: ReturnObject,
    pub primary_key: Option<PrimaryKey>,
    pub created_at: Option<CreatedAt>,
    pub updated_at: Option<UpdatedAt>,
    pub field_names: FieldNames,
    pub operations: Operations,
}

impl Attr {
    pub fn parse(input: DeriveInput) -> Self {
        let struct_name = input.ident;
        let (table_name, return_object, operations) = Parser::parse_struct_macro_arguments(&struct_name, &input.attrs);
        let (primary_key, created_at, updated_at, field_names) = Parser::parse_fields_macro_arguments(input.data);

        Attr {
            struct_name,
            table_name,
            return_object,
            primary_key,
            field_names,
            created_at,
            updated_at,
            operations,
        }
    }
}

struct Parser();

impl Parser {
    fn parse_struct_macro_arguments(struct_name: &Ident, attrs: &[Attribute]) -> (TableName, ReturnObject, Operations) {
        let mut only: Option<Vec<Operation>> = None;
        let mut exclude: Option<Vec<Operation>> = None;
        let mut return_object: Option<Ident> = None;
        let mut table_name: Option<String> = None;

        for attr in attrs {
            if attr.path().is_ident(NAME_MACRO_OPERATION_ARG) {
                let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap();
                for meta in nested {
                    match meta {
                        Meta::NameValue(name_value) if name_value.path.is_ident("table_name") => {
                            if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = name_value.clone().value {
                                table_name = Some(lit_str.value().clone());
                            };
                        },
                        Meta::NameValue(name_value) if name_value.path.is_ident("return_object") => {
                            if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = name_value.clone().value {
                                return_object = Some(
                                    parse_str::<Ident>(&lit_str.value())
                                        .expect("Failed to parse return_object as identifier")
                                );
                            };
                        },
                        Meta::NameValue(name_value) if name_value.path.is_ident("only") => {
                            if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = name_value.clone().value {
                                only = Some(
                                    lit_str.value()
                                        .split(',')
                                        .map(|s| {
                                            Operation::from_str(s.trim())
                                            .unwrap_or_else(|_| panic!("Operation {s} is not supported"))
                                        })
                                        .collect(),
                                );
                            };
                        },
                        Meta::NameValue(name_value) if name_value.path.is_ident("exclude") => {
                            if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = name_value.clone().value {
                                exclude = Some(
                                    lit_str.value()
                                        .split(',')
                                        .map(|s| {
                                            Operation::from_str(s.trim())
                                                .unwrap_or_else(|_| panic!("Operation {s} is not supported"))
                                        })
                                        .collect(),
                                );
                            };
                        },
                        _ => {
                            panic!("Error - Skip unknown name value");
                        }
                    }
                };
            }
        }

        let table_name = TableName::new(table_name.unwrap_or(struct_name.to_string()));
        let return_object = return_object.unwrap_or(format_ident!("Self"));
        let operations = Parser::get_operations(only, exclude).expect("Cannot parse the only/exclude operations properly.");
        
        (table_name, return_object, operations)
    }

    fn parse_fields_macro_arguments(data: Data) -> (Option<PrimaryKey>, Option<CreatedAt>, Option<UpdatedAt>, FieldNames) {
        let mut primary_key: Option<PrimaryKey> = None;
        let mut created_at: Option<CreatedAt> = None;
        let mut updated_at: Option<UpdatedAt> = None;
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
                                        primary_key = Some(Column::new(
                                            field_name.clone(),
                                            field.ty.clone(),
                                        ));
                                    } else if meta.path.is_ident("created_at") {
                                        created_at = Some(Column::new(
                                            field_name.clone(),
                                            field.ty.clone(),
                                        ));
                                    } else if meta.path.is_ident("updated_at") {
                                        updated_at = Some(Column::new(
                                            field_name.clone(),
                                            field.ty.clone(),
                                        ));
                                    }
                                    Ok(())
                                })
                                .unwrap_or(());
                            }
                        }

                        // Default fallbacks
                        if field_name == "id" && primary_key.is_none() {
                            primary_key = Some(Column::new(
                                field_name.clone(),
                                field.ty.clone(),
                            ));
                        } else if field_name == "created_at" && created_at.is_none() {
                            created_at = Some(Column::new(
                                field_name.clone(),
                                field.ty.clone(),
                            ));
                        } else if field_name == "updated_at" && updated_at.is_none() {
                            updated_at = Some(Column::new(
                                field_name.clone(),
                                field.ty.clone(),
                            ));
                        }

                        field_names.push(field_name);
                    }
                }
                _ => panic!("Only named fields are supported"),
            },
            _ => panic!("Only structs are supported"),
        };
        (primary_key, created_at, updated_at, field_names)
    }

    fn get_operations(only: Option<Operations>, exclude: Option<Operations>) -> Result<Operations, &'static str> {
        match (only, exclude) {
            (None, None) => Ok(Operation::all()),
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

    mod operation_tests {
        use super::{Operation, Parser};

        #[test]
        fn test_get_them_all_by_default() {
            let mut result = Parser::get_operations(None, None).unwrap();
            result.sort();
            assert_eq!(result, vec![Operation::Get, Operation::List, Operation::Create, Operation::Update, Operation::Delete]);
        }

        #[test]
        fn test_only_and_exclude() {
            let only = Some(vec![Operation::Create]);
            let exclude = Some(vec![Operation::Delete]);
            let result = Parser::get_operations(only, exclude);
            assert!(result.is_err());
        }

        #[test]
        fn test_only_one_value() {
            let only = vec![Operation::Create];
            let result = Parser::get_operations(Some(only.clone()), None);
            assert_eq!(result.unwrap(), only);
        }
        
        #[test]
        fn test_only_multiple_values() {
            let only = vec![Operation::Update, Operation::Delete];
            let result = Parser::get_operations(Some(only.clone()), None);
            assert_eq!(result.unwrap(), only);
        }

        #[test]
        fn test_exclude_one_value() {
            let exclude = Some(vec![Operation::Create]);
            let mut result = Parser::get_operations( None, exclude).unwrap();
            result.sort();
            assert_eq!(result, vec![Operation::Get, Operation::List, Operation::Update, Operation::Delete]);
        }
        
        #[test]
        fn test_exclude_multiple_values() {
            let exclude = Some(vec![Operation::Update, Operation::Delete]);
            let mut result = Parser::get_operations(None, exclude).unwrap();
            result.sort();
            assert_eq!(result, vec![Operation::Get, Operation::List, Operation::Create]);
        }
    }

    mod parse_struct_macro_arguments {
        use quote::format_ident;
        use syn::parse_quote;

        use crate::attr::{Operation, Parser};

        #[test]
        fn test_parse_only_attribute_alone() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![
                parse_quote!(#[tiny_orm(only = "create,get")])
            ];
            let (table_name, return_object, operations) = Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(table_name.0.to_string(), "my_struct".to_string());
            assert_eq!(return_object, format_ident!("Self"));
            assert_eq!(operations, vec![Operation::Create, Operation::Get]);
        }

        #[test]
        fn test_parse_exclude_attribute_alone() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![
                parse_quote!(#[tiny_orm(exclude = "create,get")])
            ];
            let (table_name, return_object, operations) = Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(table_name.0.to_string(), "my_struct".to_string());
            assert_eq!(return_object, format_ident!("Self"));
            assert_eq!(operations, vec![Operation::List, Operation::Update, Operation::Delete]);
        }

        #[test]
        fn test_parse_table_name_attribute_alone() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![
                parse_quote!(#[tiny_orm(table_name = "custom_name")])
            ];
            let (table_name, return_object, operations) = Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(table_name.0.to_string(), "custom_name".to_string());
            assert_eq!(return_object, format_ident!("Self"));
            assert_eq!(operations, Operation::all());
        }

        #[test]
        fn test_parse_return_object_attribute_alone() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![
                parse_quote!(#[tiny_orm(return_object = "Operation")])
            ];
            let (table_name, return_object, operations) = Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(table_name.0.to_string(), "my_struct".to_string());
            assert_eq!(return_object, format_ident!("Operation"));
            assert_eq!(operations, Operation::all());
        }

        #[test]
        fn test_parse_multuple_attributes() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![
                parse_quote!(#[tiny_orm(table_name = "custom", return_object = "Operation", only = "create")])
            ];
            let (table_name, return_object, operations) = Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(table_name.0.to_string(), "custom".to_string());
            assert_eq!(return_object, format_ident!("Operation"));
            assert_eq!(operations, vec![Operation::Create]);
        }

        #[test]
        fn test_parse_multuple_attributes_withespace() {
            let struct_name = format_ident!("MyStruct");
            let attrs = vec![
                parse_quote!(#[tiny_orm(table_name = "   custom ", return_object = "   Operation  ", only = "  create   ,  delete")])
            ];
            let (table_name, return_object, operations) = Parser::parse_struct_macro_arguments(&struct_name, &attrs);
            assert_eq!(table_name.0.to_string(), "custom".to_string());
            assert_eq!(return_object, format_ident!("Operation"));
            assert_eq!(operations, vec![Operation::Create, Operation::Delete]);
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

            let (primary_key, created_at, updated_at, field_names) = Parser::parse_fields_macro_arguments(input.data);
            assert_eq!(primary_key, Some(Column::new("id".to_string(), parse_quote!(i64))));
            assert_eq!(created_at, Some(Column::new("created_at".to_string(), parse_quote!(DateTime<Utc>))));
            assert_eq!(updated_at, Some(Column::new("updated_at".to_string(), parse_quote!(DateTime<Utc>))));
            assert_eq!(field_names, vec!["id".to_string(), "created_at".to_string(), "updated_at".to_string(), "last_name".to_string()]);
        }

        #[test]
        fn test_parse_no_default_columns() {
            let input: DeriveInput = parse_quote! {
                struct Contact {
                    last_name: String,
                }
            }; 

            let (primary_key, created_at, updated_at, field_names) = Parser::parse_fields_macro_arguments(input.data);
            assert_eq!(primary_key, None);
            assert_eq!(created_at, None);
            assert_eq!(updated_at, None);
            assert_eq!(field_names, vec!["last_name".to_string()]);
        }

        #[test]
        fn test_parse_override_default_values() {
            let input: DeriveInput = parse_quote! {
                struct Contact {
                    #[tiny_orm(primary_key)]
                    custom_key: u32,
                    #[tiny_orm(created_at)]
                    inserted_at: DateTime<Utc>,
                    #[tiny_orm(updated_at)]
                    something_at: DateTime<Utc>,
                    last_name: String,
                }
            }; 

            let (primary_key, created_at, updated_at, field_names) = Parser::parse_fields_macro_arguments(input.data);
            assert_eq!(primary_key, Some(Column::new("custom_key".to_string(), parse_quote!(u32))));
            assert_eq!(created_at, Some(Column::new("inserted_at".to_string(), parse_quote!(DateTime<Utc>))));
            assert_eq!(updated_at, Some(Column::new("something_at".to_string(), parse_quote!(DateTime<Utc>))));
            assert_eq!(field_names, vec!["custom_key".to_string(), "inserted_at".to_string(), "something_at".to_string(), "last_name".to_string()]);
        }

        #[test]
        fn test_parse_keep_override_even_when_default_values_present() {
            let input: DeriveInput = parse_quote! {
                struct Contact {
                    #[tiny_orm(primary_key)]
                    custom_key: u32,
                    #[tiny_orm(created_at)]
                    inserted_at: DateTime<Utc>,
                    #[tiny_orm(updated_at)]
                    something_at: DateTime<Utc>,
                    id: u32,
                    created_at: DateTime<Utc>,
                    updated_at: DateTime<Utc>,
                    last_name: String,
                }
            }; 

            let (primary_key, created_at, updated_at, field_names) = Parser::parse_fields_macro_arguments(input.data);
            assert_eq!(primary_key, Some(Column::new("custom_key".to_string(), parse_quote!(u32))));
            assert_eq!(created_at, Some(Column::new("inserted_at".to_string(), parse_quote!(DateTime<Utc>))));
            assert_eq!(updated_at, Some(Column::new("something_at".to_string(), parse_quote!(DateTime<Utc>))));
            assert_eq!(field_names, vec!["custom_key".to_string(), "inserted_at".to_string(), "something_at".to_string(),"id".to_string(), "created_at".to_string(), "updated_at".to_string(), "last_name".to_string()]);
        }
    }

    mod parse {
        use quote::format_ident;
        use syn::{parse_quote, DeriveInput};

        use crate::attr::{Column, Operation, TableName};

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
            assert_eq!(result, Attr {
                struct_name: format_ident!("Contact"),
                table_name: TableName::new("contact".to_string()),
                return_object: format_ident!("Self"),
                primary_key: Some(Column::new("id".to_string(), parse_quote!(i64))),
                created_at: Some(Column::new("created_at".to_string(), parse_quote!(DateTime<Utc>))),
                updated_at: Some(Column::new("updated_at".to_string(), parse_quote!(DateTime<Utc>))),
                field_names: vec!["id".to_string(), "created_at".to_string(), "updated_at".to_string(), "last_name".to_string()],
                operations: Operation::all(), 
            });
        }

        #[test]
        fn test_parse_all_options_specified() {
            let input: DeriveInput = parse_quote! {
                #[tiny_orm(table_name = "specific_table", only = "create", return_object = "AnotherObject")]
                struct Contact {
                    #[tiny_orm(primary_key)]
                    custom_pk: i64,
                    #[tiny_orm(created_at)]
                    custom_created_at: DateTime<Utc>,
                    #[tiny_orm(updated_at)]
                    custom_updated_at: DateTime<Utc>,
                    last_name: String,
                }
            };

            let result = Attr::parse(input);
            assert_eq!(result, Attr {
                struct_name: format_ident!("Contact"),
                table_name: TableName::new("specific_table".to_string()),
                return_object: format_ident!("AnotherObject"),
                primary_key: Some(Column::new("custom_pk".to_string(), parse_quote!(i64))),
                created_at: Some(Column::new("custom_created_at".to_string(), parse_quote!(DateTime<Utc>))),
                updated_at: Some(Column::new("custom_updated_at".to_string(), parse_quote!(DateTime<Utc>))),
                field_names: vec!["custom_pk".to_string(), "custom_created_at".to_string(), "custom_updated_at".to_string(), "last_name".to_string()],
                operations: vec![Operation::Create], 
            });
        }
    }
}
