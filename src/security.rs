//! Security context.
use std::collections::HashSet;
use std::iter::FromIterator;
use std::sync::{Arc, RwLock};

use tide::error::StringError;
use tide::Context;

const MIDDLEWARE_MISSING_MSG: &str = "SecurityMiddleware must be set";

/// Security subject.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecuritySubject {
    principal: String,
    authorities: HashSet<String>,
}

impl SecuritySubject {
    pub fn new<T: Into<String>>(principal: T, authorities: Vec<String>) -> Self {
        Self {
            principal: principal.into(),
            authorities: HashSet::from_iter(authorities),
        }
    }

    pub fn principal(&self) -> &str {
        &self.principal
    }

    pub fn has_authority(&self, authority: &str) -> bool {
        self.authorities.contains(authority)
    }
}

/// Security context.
#[derive(Debug)]
pub(crate) struct SecurityContext {
    inner: Arc<RwLock<SecurityContextInner>>,
}

impl SecurityContext {
    pub fn new(s: Option<SecuritySubject>) -> Self {
        let inner = SecurityContextInner {
            subject: s,
            changed: false,
        };
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub fn subject(&self) -> Option<SecuritySubject> {
        self.inner.read().unwrap().subject.as_ref().cloned()
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
    subject: Option<SecuritySubject>,
    changed: bool,
}

/// An extension to `Context` that provides security context.
pub trait SecurityExt {
    /// Get current subject.
    fn subject(&mut self) -> Result<Option<SecuritySubject>, StringError>;

    /// Get current principal.
    fn principal(&mut self) -> Result<Option<String>, StringError>;

    /// Check authority.
    fn check_authority(&mut self, authority: &str) -> Result<bool, StringError>;

    /// Remember principal and authorities.
    fn remember<T: Into<String>>(
        &mut self,
        principal: T,
        authorities: Vec<String>,
    ) -> Result<(), StringError>;

    fn forget(&mut self) -> Result<(), StringError>;
}

impl<AppData> SecurityExt for Context<AppData> {
    fn subject(&mut self) -> Result<Option<SecuritySubject>, StringError> {
        let sc = self
            .extensions()
            .get::<SecurityContext>()
            .ok_or_else(|| StringError(MIDDLEWARE_MISSING_MSG.to_owned()))?;

        let locked_inner = sc
            .inner
            .read()
            .map_err(|e| StringError(format!("Failed to get read lock: {}", e)))?;

        Ok(locked_inner.subject.as_ref().cloned())
    }

    fn principal(&mut self) -> Result<Option<String>, StringError> {
        let sc = self
            .extensions()
            .get::<SecurityContext>()
            .ok_or_else(|| StringError(MIDDLEWARE_MISSING_MSG.to_owned()))?;

        let locked_inner = sc
            .inner
            .read()
            .map_err(|e| StringError(format!("Failed to get read lock: {}", e)))?;

        if let Some(ref subject) = locked_inner.subject {
            Ok(Some(subject.principal().to_owned()))
        } else {
            Ok(None)
        }
    }

    fn check_authority(&mut self, authority: &str) -> Result<bool, StringError> {
        let sc = self
            .extensions()
            .get::<SecurityContext>()
            .ok_or_else(|| StringError(MIDDLEWARE_MISSING_MSG.to_owned()))?;

        let locked_inner = sc
            .inner
            .read()
            .map_err(|e| StringError(format!("Failed to get read lock: {}", e)))?;

        if let Some(ref subject) = locked_inner.subject {
            return Ok(subject.has_authority(authority));
        }

        Ok(false)
    }

    fn remember<T: Into<String>>(
        &mut self,
        principal: T,
        authorities: Vec<String>,
    ) -> Result<(), StringError> {
        let sc = self
            .extensions()
            .get::<SecurityContext>()
            .ok_or_else(|| StringError(MIDDLEWARE_MISSING_MSG.to_owned()))?;

        let mut locked_inner = sc
            .inner
            .write()
            .map_err(|e| StringError(format!("Failed to get write lock: {}", e)))?;

        locked_inner.subject = Some(SecuritySubject::new(principal.into(), authorities));
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

        if locked_inner.subject.is_some() {
            locked_inner.subject = None;
            locked_inner.changed = true;
        };

        Ok(())
    }
}
