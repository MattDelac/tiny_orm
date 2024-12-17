use sqlx::decode::Decode;
use sqlx::error::BoxDynError;
use sqlx::{Database, Type, ValueRef};

/// tiny_orm::SetOption is an enum that behave similarly to `Option` in the sense that there are only two variants.
/// The goal is to easily differentiate between an Option type and a SetOption type.
/// So that it is possible to have a struct like the following
/// ```rust
/// # use tiny_orm_core::SetOption;
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
/// # use tiny_orm_core::SetOption;
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
/// # use tiny_orm_core::SetOption;
/// let set_option: SetOption<i32> = 1.into();
/// let result: Result<i32, _> = set_option.into();
/// assert_eq!(result, Ok(1));
/// ```
/// ```rust
/// # use tiny_orm_core::SetOption;
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
    /// This return an Option<T> type wher Some<T> is corresponds when the value
    /// was using the `Set` variant,
    ///
    /// # Examples
    /// ```rust
    /// # use tiny_orm_core::SetOption;
    /// let set = SetOption::Set(1);
    /// let inner = set.inner();
    /// assert_eq!(inner, Some(1));
    /// ```
    ///
    /// ```rust
    /// # use tiny_orm_core::SetOption;
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

    /// `is_set()` returns true if the variant is `Set`
    ///
    /// # Examples
    /// ```rust
    /// # use tiny_orm_core::SetOption;
    /// let set = SetOption::Set(1);
    /// assert!(set.is_set());
    /// ```
    ///
    /// ```rust
    /// # use tiny_orm_core::SetOption;
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
    /// # use tiny_orm_core::SetOption;
    /// let set = SetOption::Set(1);
    /// assert!(!set.is_not_set());
    /// ```
    ///
    /// ```rust
    /// # use tiny_orm_core::SetOption;
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

/// Implements database decoding for SetOption<T>.
/// This allows automatic conversion from database values to SetOption<T>.
///
/// # Examples
/// ```rust
/// # use sqlx::SqlitePool;
/// # use tiny_orm_core::SetOption;
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
