//! Lusion Validation.
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
#[macro_use]
extern crate assert_matches;

mod error;
mod length;

pub use self::error::{ValidationError, ValidationErrors};
pub use self::length::{HasLength, Length, LengthValidator};

/// Validation a struct.
///
/// # Examples
///
/// ```rust
/// use lusion_validator::{validate, Length};
///
/// struct User {
///     username: String,
///     password: String,
/// }
///
/// let user = User {
///     username: "user".to_owned(),
///     password: "1234".to_owned(),
/// };
///
/// let errors = validate!(user, {
///     username: [Length(Some(1), Some(20))],
///     password: [Length(Some(1), Some(20))],
/// });
///
/// assert!(errors.is_empty());
/// ```
#[macro_export]
macro_rules! validate {
    ($val:expr, {
        $($field:ident: [$($validator:expr),+]),+ $(,)*
    }) => ({
        use $crate::{ValidationErrors, Validator};

        let mut errors = ValidationErrors::new();

        $(
            $(
                if let Some(error) = $validator.check(&$val.$field) {
                    errors.entry(stringify!($field))
                        .or_insert_with(|| Vec::new())
                        .push(error);
                };
            )+
        )+

        errors
    });
}

/// A `Validator` trait for check `T`
pub trait Validator<T> {
    fn check(&self, val: &T) -> Option<ValidationError>;
}

impl<T, V> Validator<Option<T>> for V
where
    V: Validator<T>,
{
    fn check(&self, value: &Option<T>) -> Option<ValidationError> {
        match *value {
            Some(ref value) => self.check(value),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_for_option() {
        struct JustErrorValidator;

        impl Validator<()> for JustErrorValidator {
            fn check(&self, val: &()) -> Option<ValidationError> {
                Some(ValidationError::new("just_error", val, &Vec::<()>::new()))
            }
        }

        let value = ();
        let error = JustErrorValidator.check(&value);

        assert_matches!(error, Some(err) => {
            assert_eq!(err, ValidationError::new("just_error", &value, &Vec::<()>::new()));
        });

        let error = JustErrorValidator.check(&Option::<()>::None);
        assert_matches!(error, None);
    }

    #[test]
    fn test_validate_macro() {
        struct User {
            username: String,
            password: String,
        }

        let user = User {
            username: "user".to_owned(),
            password: "1234".to_owned(),
        };

        let errors = validate!(user, {
            username: [Length(Some(1), Some(20))],
            password: [Length(Some(1), Some(20))],
        });

        assert!(errors.is_empty());
    }
}
