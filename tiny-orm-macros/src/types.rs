use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::format_ident;
use std::{fmt, str::FromStr};
use syn::{Ident, Type};

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
    pub fn remove_prefix(&self, input: String) -> String {
        match self {
            StructType::Create => input.replace("New", ""),
            StructType::Update => input.replace("Update", ""),
            StructType::Generic => input,
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
            None => struct_type.remove_prefix(name.clone()),
        };

        let return_object = match (return_object, &struct_type) {
            (Some(value), _) => value,
            (None, &StructType::Generic) => format_ident!("Self"),
            (None, _) => format_ident!("{}", struct_type.remove_prefix(name)),
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
}
impl Column {
    pub fn new(name: String, _type: Type) -> Self {
        Self {
            name: name.clone(),
            ident: format_ident!("{}", name),
            _type,
            auto_increment: false,
        }
    }
    pub fn set_auto_increment(&mut self) {
        self.auto_increment = true;
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
pub type FieldNames = Vec<String>;
pub type Operations = Vec<Operation>;

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_set_auto_increment() {
        let mut column = Column::new("col_name".to_string(), parse_quote!(i32));
        assert!(!column.auto_increment);
        column.set_auto_increment();
        assert!(column.auto_increment);
    }
}
