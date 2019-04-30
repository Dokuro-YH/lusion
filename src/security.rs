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
pub struct SecurityContext {
    inner: Arc<RwLock<SecurityContextInner>>,
}

impl SecurityContext {
    pub fn new(s: Option<SecuritySubject>) -> Self {
        let inner = SecurityContextInner::new(s);
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub fn subject(&self) -> Option<SecuritySubject> {
        self.inner.read().unwrap().subject.clone()
    }

    pub fn changed(&self) -> bool {
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

impl SecurityContextInner {
    fn new(s: Option<SecuritySubject>) -> Self {
        Self {
            subject: s,
            changed: false,
        }
    }

    fn update(&mut self, subject: SecuritySubject) {
        self.subject = Some(subject);
        self.changed = true;
    }

    fn clear(&mut self) {
        self.subject = None;
        self.changed = true;
    }
}

/// An extension to `Context` that provides security context.
pub trait SecurityExt {
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

        locked_inner.update(SecuritySubject::new(principal.into(), authorities));

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
        locked_inner.clear();

        Ok(())
    }
}
