#![cfg(feature = "set-option")]
pub mod errors;
pub use crate::errors::TinyOrmError;

/// tiny_orm::SetOption is an enum that behave similarly to `Option` in the sense that there are only two variants.
/// The goal is to easily differentiate between an Option type and a SetOption type.
/// So that it is possible to have a struct like the following
/// ```rust
/// # use tiny_orm_model::SetOption;
///
/// struct Todo {
///     id: SetOption<i64>,
///     description: SetOption<Option<String>>,
/// }
/// ```
///
/// When the variant will be `SetOption::NotSet`, then tiny ORM will automatically skip the field during "write" operations
/// like `create()` or `update()`.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SetOption<T> {
    Set(T),
    #[default]
    NotSet,
}

/// Implement `From` for `SetOption` to allow for easy conversion from a value to a `SetOption`.
/// ```rust
/// # use tiny_orm_model::SetOption;
/// let set_option: SetOption<i32> = 1.into();
/// assert_eq!(set_option, SetOption::Set(1));
/// ```
impl<T> From<T> for SetOption<T> {
    fn from(value: T) -> Self {
        SetOption::Set(value)
    }
}

/// Implement `From` for `Result` to allow for easy conversion from a `SetOption` to a `Result`.
/// This is useful when you want to handle the `NotSet` variant as an error case.
///
/// # Examples
/// ```rust
/// # use tiny_orm_model::SetOption;
/// let set_option: SetOption<i32> = 1.into();
/// let result: Result<i32, _> = set_option.into();
/// assert_eq!(result, Ok(1));
/// ```
/// ```rust
/// # use tiny_orm_model::SetOption;
/// let not_set: SetOption<i32> = SetOption::NotSet;
/// let result: Result<i32, _> = not_set.into();
/// assert_eq!(result, Err("Cannot convert NotSet variant to value"));
/// ```
impl<T> From<SetOption<T>> for Result<T, &'static str> {
    fn from(value: SetOption<T>) -> Self {
        match value {
            SetOption::Set(value) => Ok(value),
            SetOption::NotSet => Err("Cannot convert NotSet variant to value"),
        }
    }
}

impl<T> SetOption<T> {
    /// `inner()` is a method to get the inner value as an Option type.
    /// This return an `Option<T>` type where `Some<T>` corresponds to the `Set` variant,
    ///
    /// # Examples
    /// ```rust
    /// # use tiny_orm_model::SetOption;
    /// let set = SetOption::Set(1);
    /// let inner = set.inner();
    /// assert_eq!(inner, Some(1));
    /// ```
    ///
    /// ```rust
    /// # use tiny_orm_model::SetOption;
    /// let not_set: SetOption<i32> = SetOption::NotSet;
    /// let inner = not_set.inner();
    /// assert_eq!(inner, None);
    /// ```
    pub fn inner(self) -> Option<T> {
        match self {
            SetOption::NotSet => None,
            SetOption::Set(value) => Some(value),
        }
    }

    /// `value()` is a method to get the inner value as an Result type.
    /// This return an `Result<T, TinyOrmError>` type where `Ok<T>` corresponds to the `Set` variant,
    ///
    /// # Examples
    /// ```rust
    /// # use tiny_orm_model::SetOption;
    /// let set = SetOption::Set(1);
    /// let inner = set.value();
    /// assert_eq!(inner, Ok(1));
    /// ```
    ///
    /// ```rust
    /// use tiny_orm_model::{SetOption, TinyOrmError};
    /// let not_set: SetOption<i32> = SetOption::NotSet;
    /// let inner = not_set.value();
    /// assert_eq!(inner, Err(TinyOrmError::SetOptionNotSet));
    /// ```
    pub fn value(self) -> Result<T, TinyOrmError> {
        match self {
            SetOption::NotSet => Err(TinyOrmError::SetOptionNotSet),
            SetOption::Set(value) => Ok(value),
        }
    }

    /// `is_set()` returns true if the variant is `Set`
    ///
    /// # Examples
    /// ```rust
    /// # use tiny_orm_model::SetOption;
    /// let set = SetOption::Set(1);
    /// assert!(set.is_set());
    /// ```
    ///
    /// ```rust
    /// # use tiny_orm_model::SetOption;
    /// let not_set: SetOption<i32> = SetOption::NotSet;
    /// assert!(!not_set.is_set());
    /// ```
    pub fn is_set(&self) -> bool {
        match self {
            SetOption::Set(_) => true,
            SetOption::NotSet => false,
        }
    }

    /// `is_not_set()` returns true if the variant is `NotSet`
    ///
    /// # Examples
    /// ```rust
    /// # use tiny_orm_model::SetOption;
    /// let set = SetOption::Set(1);
    /// assert!(!set.is_not_set());
    /// ```
    ///
    /// ```rust
    /// # use tiny_orm_model::SetOption;
    /// let not_set: SetOption<i32> = SetOption::NotSet;
    /// assert!(not_set.is_not_set());
    /// ```
    pub fn is_not_set(&self) -> bool {
        match self {
            SetOption::Set(_) => false,
            SetOption::NotSet => true,
        }
    }
}

#[cfg(not(feature = "sqlx-0.7"))]
use sqlx::decode::Decode;
#[cfg(not(feature = "sqlx-0.7"))]
use sqlx::encode::IsNull;
#[cfg(not(feature = "sqlx-0.7"))]
use sqlx::error::BoxDynError;
#[cfg(not(feature = "sqlx-0.7"))]
use sqlx::{Database, Encode, Type, ValueRef};

/// Implements database decoding for `SetOption<T>`.
/// This allows automatic conversion from database values to `SetOption<T>`.
///
/// # Examples
/// ```rust
/// # use sqlx::SqlitePool;
/// # use tiny_orm_model::SetOption;
/// # use sqlx::FromRow;
/// #
/// #[derive(FromRow)]
/// struct TestRecord {
///     value: SetOption<i32>,
/// }
///
/// # tokio_test::block_on(async {
/// let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
///
/// # sqlx::query(
/// #    "CREATE TABLE test (value INTEGER)"
/// # ).execute(&pool).await.unwrap();
///
/// # sqlx::query(
/// #    "INSERT INTO test (value) VALUES (?)"
/// # )
/// # .bind(42)
/// # .execute(&pool).await.unwrap();
///
/// // Test with a value
/// let record: TestRecord = sqlx::query_as("SELECT value FROM test")
///     .fetch_one(&pool)
///     .await
///     .unwrap();
///
/// assert_eq!(record.value, SetOption::Set(42));
///
/// // Test NULL value
/// # sqlx::query("DELETE FROM test").execute(&pool).await.unwrap();
/// # sqlx::query(
/// #    "INSERT INTO test (value) VALUES (?)"
/// # )
/// # .bind(None::<i32>)
/// # .execute(&pool).await.unwrap();
///
/// let record: TestRecord = sqlx::query_as("SELECT value FROM test")
///     .fetch_one(&pool)
///     .await
///     .unwrap();
///
/// assert_eq!(record.value, SetOption::NotSet);
/// # });
/// ```
#[cfg(not(feature = "sqlx-0.7"))]
impl<'r, DB, T> Decode<'r, DB> for SetOption<T>
where
    DB: Database,
    T: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        if value.is_null() {
            return Ok(SetOption::NotSet);
        }

        Ok(SetOption::Set(T::decode(value)?))
    }
}

#[cfg(not(feature = "sqlx-0.7"))]
impl<DB, T> Type<DB> for SetOption<T>
where
    DB: Database,
    T: Type<DB>,
{
    fn type_info() -> <DB as Database>::TypeInfo {
        T::type_info()
    }

    fn compatible(ty: &<DB as Database>::TypeInfo) -> bool {
        T::compatible(ty)
    }
}

/// Implements database encoding for `SetOption<T>`.
/// This allows automatic conversion from `SetOption<T>` to database values.
///
/// # Examples
/// ```rust
/// # use sqlx::SqlitePool;
/// # use tiny_orm_model::SetOption;
/// # use sqlx::FromRow;
/// #
/// #[derive(FromRow)]
/// struct TestRecord {
///     value: SetOption<i32>,
/// }
///
/// # tokio_test::block_on(async {
/// let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
///
/// // Create test table
/// sqlx::query("CREATE TABLE test (value INTEGER)")
///     .execute(&pool)
///     .await
///     .unwrap();
///
/// // Test inserting a Set value
/// let set_value = SetOption::Set(42);
/// sqlx::query("INSERT INTO test (value) VALUES (?)")
///     .bind(set_value)
///     .execute(&pool)
///     .await
///     .unwrap();
///
/// let record: TestRecord = sqlx::query_as("SELECT value FROM test")
///     .fetch_one(&pool)
///     .await
///     .unwrap();
/// assert_eq!(record.value, SetOption::Set(42));
///
/// // Test inserting a NotSet value
/// sqlx::query("DELETE FROM test").execute(&pool).await.unwrap();
/// let not_set_value: SetOption<i32> = SetOption::NotSet;
/// sqlx::query("INSERT INTO test (value) VALUES (?)")
///     .bind(not_set_value)
///     .execute(&pool)
///     .await
///     .unwrap();
///
/// let record: TestRecord = sqlx::query_as("SELECT value FROM test")
///     .fetch_one(&pool)
///     .await
///     .unwrap();
/// assert_eq!(record.value, SetOption::NotSet);
/// # });
/// ```
#[cfg(not(feature = "sqlx-0.7"))]
impl<'q, T, DB: Database> Encode<'q, DB> for SetOption<T>
where
    T: Encode<'q, DB>,
{
    fn encode(self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<IsNull, BoxDynError> {
        match self {
            SetOption::Set(value) => value.encode(buf),
            SetOption::NotSet => Ok(IsNull::Yes),
        }
    }

    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        match self {
            SetOption::Set(value) => value.encode_by_ref(buf),
            SetOption::NotSet => Ok(IsNull::Yes),
        }
    }

    fn produces(&self) -> Option<DB::TypeInfo> {
        match self {
            SetOption::Set(value) => value.produces(),
            SetOption::NotSet => None,
        }
    }

    fn size_hint(&self) -> usize {
        match self {
            SetOption::Set(value) => value.size_hint(),
            SetOption::NotSet => 0,
        }
    }
}

// ##########################
// ##########################
// SQLX 0.7
// ##########################
// ##########################
#[cfg(feature = "sqlx-0.7")]
use sqlx::{encode::IsNull, error::BoxDynError, Database, ValueRef};
#[cfg(feature = "sqlx-0.7")]
impl<DB: Database, T> sqlx::Type<DB> for SetOption<T>
where
    T: sqlx::Type<DB>,
{
    fn type_info() -> <DB as Database>::TypeInfo {
        T::type_info()
    }
}

#[cfg(all(feature = "sqlx-0.7", feature = "mysql"))]
use sqlx::{mysql::MySqlValueRef, MySql};
#[cfg(all(feature = "sqlx-0.7", feature = "postgres"))]
use sqlx::{
    postgres::{PgArgumentBuffer, PgValueRef},
    Postgres,
};
#[cfg(all(feature = "sqlx-0.7", feature = "sqlite"))]
use sqlx::{
    sqlite::{SqliteArgumentValue, SqliteValueRef},
    Sqlite,
};

/// Implements database decoding for SetOption<T>.
/// This allows automatic conversion from database values to SetOption<T>.
///
/// # Examples
/// ```rust
/// # use sqlx::SqlitePool;
/// # use tiny_orm_model::SetOption;
/// # use sqlx::FromRow;
/// #
/// #[derive(FromRow)]
/// struct TestRecord {
///     value: SetOption<i32>,
/// }
///
/// # tokio_test::block_on(async {
/// let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
///
/// # sqlx::query(
/// #    "CREATE TABLE test (value INTEGER)"
/// # ).execute(&pool).await.unwrap();
///
/// # sqlx::query(
/// #    "INSERT INTO test (value) VALUES (?)"
/// # )
/// # .bind(42)
/// # .execute(&pool).await.unwrap();
///
/// // Test with a value
/// let record: TestRecord = sqlx::query_as("SELECT value FROM test")
///     .fetch_one(&pool)
///     .await
///     .unwrap();
///
/// assert_eq!(record.value, SetOption::Set(42));
///
/// // Test NULL value
/// # sqlx::query("DELETE FROM test").execute(&pool).await.unwrap();
/// # sqlx::query(
/// #    "INSERT INTO test (value) VALUES (?)"
/// # )
/// # .bind(None::<i32>)
/// # .execute(&pool).await.unwrap();
///
/// let record: TestRecord = sqlx::query_as("SELECT value FROM test")
///     .fetch_one(&pool)
///     .await
///     .unwrap();
///
/// assert_eq!(record.value, SetOption::NotSet);
/// # });
/// ```
#[cfg(all(feature = "sqlx-0.7", feature = "sqlite"))]
impl<'r, T> sqlx::Decode<'r, Sqlite> for SetOption<T>
where
    T: sqlx::Decode<'r, Sqlite>,
{
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        if value.is_null() {
            return Ok(SetOption::NotSet);
        }
        let decoded = <T as sqlx::Decode<'r, Sqlite>>::decode(value)?;
        Ok(SetOption::Set(decoded))
    }
}
#[cfg(all(feature = "sqlx-0.7", feature = "postgres"))]
impl<'r, T> sqlx::Decode<'r, Postgres> for SetOption<T>
where
    T: sqlx::Decode<'r, Postgres>,
{
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        if value.is_null() {
            return Ok(SetOption::NotSet);
        }
        let decoded = <T as sqlx::Decode<'r, Postgres>>::decode(value)?;
        Ok(SetOption::Set(decoded))
    }
}
#[cfg(all(feature = "sqlx-0.7", feature = "mysql"))]
impl<'r, T> sqlx::Decode<'r, MySql> for SetOption<T>
where
    T: sqlx::Decode<'r, MySql>,
{
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        if value.is_null() {
            return Ok(SetOption::NotSet);
        }
        let decoded = <T as sqlx::Decode<'r, MySql>>::decode(value)?;
        Ok(SetOption::Set(decoded))
    }
}

/// Implements database encoding for SetOption<T>.
/// This allows automatic conversion from SetOption<T> to database values.
///
/// # Examples
/// ```rust
/// # use sqlx::SqlitePool;
/// # use tiny_orm_model::SetOption;
/// # use sqlx::FromRow;
/// #
/// #[derive(FromRow)]
/// struct TestRecord {
///     value: SetOption<i32>,
/// }
///
/// # tokio_test::block_on(async {
/// let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
///
/// // Create test table
/// sqlx::query("CREATE TABLE test (value INTEGER)")
///     .execute(&pool)
///     .await
///     .unwrap();
///
/// // Test inserting a Set value
/// let set_value = SetOption::Set(42);
/// sqlx::query("INSERT INTO test (value) VALUES (?)")
///     .bind(set_value)
///     .execute(&pool)
///     .await
///     .unwrap();
///
/// let record: TestRecord = sqlx::query_as("SELECT value FROM test")
///     .fetch_one(&pool)
///     .await
///     .unwrap();
/// assert_eq!(record.value, SetOption::Set(42));
///
/// // Test inserting a NotSet value
/// sqlx::query("DELETE FROM test").execute(&pool).await.unwrap();
/// let not_set_value: SetOption<i32> = SetOption::NotSet;
/// sqlx::query("INSERT INTO test (value) VALUES (?)")
///     .bind(not_set_value)
///     .execute(&pool)
///     .await
///     .unwrap();
///
/// let record: TestRecord = sqlx::query_as("SELECT value FROM test")
///     .fetch_one(&pool)
///     .await
///     .unwrap();
/// assert_eq!(record.value, SetOption::NotSet);
/// # });
/// ```
#[cfg(all(feature = "sqlx-0.7", feature = "sqlite"))]
impl<'q, T> sqlx::Encode<'q, Sqlite> for SetOption<T>
where
    T: sqlx::Encode<'q, Sqlite>,
{
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'q>>) -> IsNull {
        match self {
            SetOption::Set(value) => value.encode_by_ref(args),
            SetOption::NotSet => IsNull::Yes,
        }
    }
}
#[cfg(all(feature = "sqlx-0.7", feature = "postgres"))]
impl<'q, T> sqlx::Encode<'q, Postgres> for SetOption<T>
where
    T: sqlx::Encode<'q, Postgres>,
{
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        match self {
            SetOption::Set(value) => value.encode_by_ref(buf),
            SetOption::NotSet => IsNull::Yes,
        }
    }
}
#[cfg(all(feature = "sqlx-0.7", feature = "mysql"))]
impl<'q, T> sqlx::Encode<'q, MySql> for SetOption<T>
where
    T: sqlx::Encode<'q, MySql>,
{
    fn encode_by_ref(&self, args: &mut Vec<u8>) -> IsNull {
        match self {
            SetOption::Set(value) => value.encode_by_ref(args),
            SetOption::NotSet => IsNull::Yes,
        }
    }
}
