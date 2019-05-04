use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use super::{ValidationError, Validator};

/// Create a `LengthValidator`, validate to implement the `HasLength` trait object.
#[allow(non_snake_case)]
pub fn Length(min: Option<usize>, max: Option<usize>) -> LengthValidator {
    LengthValidator(min, max)
}

pub struct LengthValidator(Option<usize>, Option<usize>);

impl<T> Validator<T> for LengthValidator
where
    T: HasLength,
{
    fn validate(&self, value: &T) -> Option<ValidationError> {
        match (self.0, self.1) {
            (Some(min), Some(max)) if min > value.length() || value.length() > max => {
                Some(ValidationError::with_params("length", &[min, max]))
            }
            (Some(min), None) if min > value.length() => {
                Some(ValidationError::with_params("min_length", &[min]))
            }
            (None, Some(max)) if value.length() > max => {
                Some(ValidationError::with_params("max_length", &[max]))
            }
            _ => None,
        }
    }
}

pub trait HasLength {
    fn length(&self) -> usize;
}

impl<'a> HasLength for &'a str {
    fn length(&self) -> usize {
        self.len()
    }
}

impl HasLength for String {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<'a> HasLength for Cow<'a, str> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for Vec<T> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<K, V> HasLength for HashMap<K, V> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<V> HasLength for HashSet<V> {
    fn length(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_validator_error {
        ([$($value:expr),*], $code:expr, min: $min:expr, max: $max:expr) => (
            let validator = LengthValidator(Some($min), Some($max));
            $(
                let error = validator.validate($value);
                assert_matches!(error, Some(err) => {
                    assert_eq!(err, ValidationError::with_params($code, &vec![$min, $max]));
                });
            )*
        );
        ([$($value:expr),*], $code:expr, min: $min:expr) => (
            let validator = LengthValidator(Some($min), None);
            $(
                let error = validator.validate($value);
                assert_matches!(error, Some(err) => {
                    assert_eq!(err, ValidationError::with_params($code, &vec![$min]));
                });
            )*
        );
        ([$($value:expr),*], $code:expr, max: $max:expr) => (
            let validator = LengthValidator(None, Some($max));
            $(
                let error = validator.validate($value);
                assert_matches!(error, Some(err) => {
                    assert_eq!(err, ValidationError::with_params($code, &vec![$max]));
                });
            )*
        );
    }

    #[test]
    fn test_length_validator_with_str() {
        let empty: &'static str = "";
        let long: &'static str = "123456";
        assert_validator_error!([&empty, &long], "length", min: 1, max: 4);
        assert_validator_error!([&empty], "min_length", min: 1);
        assert_validator_error!([&long], "max_length", max: 4);
    }

    #[test]
    fn test_length_validator_with_string() {
        let empty = "".to_owned();
        let long = "123456".to_owned();
        assert_validator_error!([&empty, &long], "length", min: 1, max: 4);
        assert_validator_error!([&empty], "min_length", min: 1);
        assert_validator_error!([&long], "max_length", max: 4);
    }

    #[test]
    fn test_length_validator_with_cow() {
        let empty = Cow::from("");
        let long = Cow::from("123456");
        assert_validator_error!([&empty, &long], "length", min: 1, max: 4);
        assert_validator_error!([&empty], "min_length", min: 1);
        assert_validator_error!([&long], "max_length", max: 4);
    }

    #[test]
    fn test_length_validator_with_vec() {
        let empty = Vec::<usize>::new();
        let long = std::iter::repeat(1).take(10).collect::<Vec<usize>>();
        assert_validator_error!([&empty, &long], "length", min: 1, max: 4);
        assert_validator_error!([&empty], "min_length", min: 1);
        assert_validator_error!([&long], "max_length", max: 4);
    }

    #[test]
    fn test_length_validator_with_hashset() {
        let empty = HashSet::<usize>::new();
        let long = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]
            .iter()
            .cloned()
            .collect::<HashSet<usize>>();
        assert_validator_error!([&empty, &long], "length", min: 1, max: 4);
        assert_validator_error!([&empty], "min_length", min: 1);
        assert_validator_error!([&long], "max_length", max: 4);
    }

    #[test]
    fn test_length_validator_with_hashmap() {
        let empty = HashMap::<usize, usize>::new();
        let long = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]
            .into_iter()
            .map(|s| (s, s))
            .collect::<HashMap<usize, usize>>();
        assert_validator_error!([&empty, &long], "length", min: 1, max: 4);
        assert_validator_error!([&empty], "min_length", min: 1);
        assert_validator_error!([&long], "max_length", max: 4);
    }
}
