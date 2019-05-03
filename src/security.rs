//! Security context.
use std::sync::{Arc, RwLock};

use tide::error::StringError;
use tide::Context;

const MIDDLEWARE_MISSING_MSG: &str = "SecurityMiddleware must be set";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Identity(String);

impl Identity {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Identity(s.into())
    }
}

/// Security context.
#[derive(Debug)]
pub(crate) struct SecurityContext {
    inner: Arc<RwLock<SecurityContextInner>>,
}

impl SecurityContext {
    pub fn new(identity: Option<Identity>) -> Self {
        let inner = SecurityContextInner {
            identity,
            changed: false,
        };
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub fn identity(&self) -> Option<Identity> {
        self.inner.read().unwrap().identity.clone()
    }

    pub fn is_changed(&self) -> bool {
        self.inner.read().unwrap().changed
    }
}

impl Clone for SecurityContext {
    fn clone(&self) -> Self {
        let inner = Arc::clone(&self.inner);
        Self { inner }
    }
}

#[derive(Debug)]
struct SecurityContextInner {
    identity: Option<Identity>,
    changed: bool,
}

/// An extension to `Context` that provides security context.
pub trait SecurityExt {
    /// Get current identity.
    fn identity(&mut self) -> Result<Option<Identity>, StringError>;

    /// Remember principal and authorities.
    fn remember(&mut self, identity: Identity) -> Result<(), StringError>;

    fn forget(&mut self) -> Result<(), StringError>;
}

impl<AppData> SecurityExt for Context<AppData> {
    fn identity(&mut self) -> Result<Option<Identity>, StringError> {
        let sc = self
            .extensions()
            .get::<SecurityContext>()
            .ok_or_else(|| StringError(MIDDLEWARE_MISSING_MSG.to_owned()))?;

        let locked_inner = sc
            .inner
            .read()
            .map_err(|e| StringError(format!("Failed to get read lock: {}", e)))?;

        Ok(locked_inner.identity.clone())
    }

    fn remember(&mut self, identity: Identity) -> Result<(), StringError> {
        let sc = self
            .extensions()
            .get::<SecurityContext>()
            .ok_or_else(|| StringError(MIDDLEWARE_MISSING_MSG.to_owned()))?;

        let mut locked_inner = sc
            .inner
            .write()
            .map_err(|e| StringError(format!("Failed to get write lock: {}", e)))?;

        locked_inner.identity = Some(identity);
        locked_inner.changed = true;

        Ok(())
    }

    fn forget(&mut self) -> Result<(), StringError> {
        let sc = self
            .extensions()
            .get::<SecurityContext>()
            .ok_or_else(|| StringError(MIDDLEWARE_MISSING_MSG.to_owned()))?;

        let mut locked_inner = sc
            .inner
            .write()
            .map_err(|e| StringError(format!("Failed to get write lock: {}", e)))?;

        if locked_inner.identity.is_some() {
            locked_inner.identity = None;
            locked_inner.changed = true;
        };

        Ok(())
    }
}
