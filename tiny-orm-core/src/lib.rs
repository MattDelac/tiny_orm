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
        let var_name = match self {
            SetOption::Set(_) => true,
            SetOption::NotSet => false,
        };
        var_name
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
