use convert_case::{Case, Casing};
use once_cell::sync::Lazy;
use quote::{format_ident, ToTokens};
use regex::Regex;
use std::{fmt, str::FromStr};
use syn::{Ident, Type};

static FIND_SET_OPTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:tiny_orm\s*::\s*)*SetOption\s*<").unwrap());

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructType {
    Create,
    Update,
    Generic,
}
impl StructType {
    pub fn default_operation(&self) -> Operations {
        match self {
            StructType::Create => vec![Operation::Create],
            StructType::Update => vec![Operation::Update],
            StructType::Generic => vec![Operation::Get, Operation::List, Operation::Delete],
        }
    }
    pub fn remove_prefix(&self, input: &str) -> String {
        match self {
            StructType::Create => input.to_string().replace("New", ""),
            StructType::Update => input.to_string().replace("Update", ""),
            StructType::Generic => input.to_string(),
        }
    }
}
impl From<&str> for StructType {
    fn from(input: &str) -> StructType {
        if input.starts_with("New") {
            StructType::Create
        } else if input.starts_with("Update") {
            StructType::Update
        } else {
            StructType::Generic
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStruct {
    pub name: StructName,
    pub table_name: TableName,
    pub struct_type: StructType,
    pub return_object: ReturnObject,
}
impl ParsedStruct {
    pub fn new(
        struct_name: &Ident,
        table_name: Option<String>,
        return_object: Option<ReturnObject>,
    ) -> Self {
        let name = struct_name.to_string();
        let struct_type = StructType::from(name.as_str());

        let table_name = match table_name {
            Some(value) => value,
            None => struct_type.remove_prefix(&name),
        };

        let return_object = match (return_object, &struct_type) {
            (Some(value), _) => value,
            (None, &StructType::Generic) => format_ident!("Self"),
            (None, _) => format_ident!("{}", struct_type.remove_prefix(&name)),
        };

        Self {
            name: struct_name.clone(),
            table_name: TableName::new(table_name),
            struct_type,
            return_object,
        }
    }
}

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
        vec![
            Operation::Get,
            Operation::List,
            Operation::Create,
            Operation::Update,
            Operation::Delete,
        ]
    }
}

impl FromStr for Operation {
    type Err = ();

    fn from_str(input: &str) -> Result<Operation, Self::Err> {
        match input {
            "get" => Ok(Operation::Get),
            "list" => Ok(Operation::List),
            "create" => Ok(Operation::Create),
            "update" => Ok(Operation::Update),
            "delete" => Ok(Operation::Delete),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub ident: Ident,
    pub _type: Type,
    pub auto_increment: bool,
    pub primary_key: bool,
}
impl Column {
    pub fn new(name: &str, _type: Type) -> Self {
        Self {
            name: name.to_string(),
            ident: format_ident!("{}", name),
            _type,
            auto_increment: false,
            primary_key: false,
        }
    }
    pub fn set_auto_increment(&mut self) {
        self.auto_increment = true;
    }
    pub fn set_primary_key(&mut self) {
        self.primary_key = true;
    }
    pub fn use_set_options(&self) -> bool {
        FIND_SET_OPTION_REGEX.is_match(&self._type.to_token_stream().to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableName(pub String);
impl TableName {
    pub fn new(input: String) -> Self {
        Self(input.to_case(Case::Snake))
    }
}

impl fmt::Display for TableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type StructName = Ident;
pub type PrimaryKey = Column;
pub type ReturnObject = Ident;
pub type Operations = Vec<Operation>;

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_set_auto_increment() {
        let mut column = Column::new("col_name", parse_quote!(i32));
        assert!(!column.auto_increment);
        column.set_auto_increment();
        assert!(column.auto_increment);
    }

    mod column {
        use super::*;

        #[test]
        fn test_use_set_options_true() {
            let col_name = "col_name";
            assert!(
                Column::new(col_name, parse_quote!(tiny_orm::SetOption<i32>)).use_set_options()
            );
            assert!(Column::new(col_name, parse_quote!(SetOption<String>)).use_set_options());
            assert!(Column::new(col_name, parse_quote!(SetOption<!>)).use_set_options());
            assert!(Column::new(col_name, parse_quote!(SetOption<Option<bool>>)).use_set_options());
        }
        #[test]
        fn test_use_set_options_false() {
            let col_name = "col_name";
            assert!(!Column::new(col_name, parse_quote!(Option<i32>)).use_set_options());
            assert!(!Column::new(col_name, parse_quote!(String)).use_set_options());
            assert!(!Column::new(col_name, parse_quote!(!)).use_set_options());
            assert!(!Column::new(col_name, parse_quote!(bool)).use_set_options());
            assert!(!Column::new(col_name, parse_quote!(MyStruct<SetOption>)).use_set_options());
            assert!(!Column::new(col_name, parse_quote!(Option<SetOption>)).use_set_options());
            assert!(
                !Column::new(col_name, parse_quote!(Option<SetOption<bool>>)).use_set_options()
            );
        }
    }
}
