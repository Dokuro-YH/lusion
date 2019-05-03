use std::borrow::Cow;
use std::collections::HashMap;

pub type ValidationErrors = HashMap<&'static str, Vec<ValidationError>>;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ValidationError {
    code: Cow<'static, str>,
    value: serde_json::Value,
    params: Vec<serde_json::Value>,
}

impl ValidationError {
    pub fn new<T: serde::Serialize, P: serde::Serialize>(
        code: &'static str,
        value: &T,
        params: &[P],
    ) -> Self {
        ValidationError {
            code: Cow::from(code),
            value: serde_json::to_value(value).unwrap(),
            params: params
                .iter()
                .map(|p| serde_json::to_value(p).unwrap())
                .collect(),
        }
    }
}
