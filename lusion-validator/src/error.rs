use std::borrow::Cow;
use std::collections::HashMap;

pub type ValidationErrors = HashMap<&'static str, Vec<ValidationError>>;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ValidationError {
    code: Cow<'static, str>,
    params: Vec<serde_json::Value>,
}

impl ValidationError {
    pub fn new(code: &'static str) -> Self {
        ValidationError {
            code: Cow::from(code),
            params: Vec::new(),
        }
    }

    pub fn with_params<P: serde::Serialize>(code: &'static str, params: &[P]) -> Self {
        ValidationError {
            code: Cow::from(code),
            params: params
                .iter()
                .map(|p| serde_json::to_value(p).unwrap())
                .collect(),
        }
    }

    pub fn param<P: serde::Serialize>(&mut self, param: P) -> &mut Self {
        self.params.push(serde_json::to_value(param).unwrap());
        self
    }
}
