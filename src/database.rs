use quote::format_ident;
use syn::Ident;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DbType {
    Postgres,
    MySQL,
    Sqlite,
}
impl DbType {
    pub fn to_ident(&self) -> Ident {
        match self {
            DbType::Postgres => format_ident!("Postgres"),
            DbType::MySQL => format_ident!("MySql"),
            DbType::Sqlite => format_ident!("Sqlite"),
        }
    }
}

#[cfg(feature = "postgres")]
pub fn db_type() -> DbType {
    DbType::Postgres
}

#[cfg(feature = "mysql")]
pub fn db_type() -> DbType {
    DbType::MySQL
}

#[cfg(feature = "sqlite")]
pub fn db_type() -> DbType {
    DbType::Sqlite
}
