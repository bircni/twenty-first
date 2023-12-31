/// Indicates the lock's underlying type
#[derive(Debug, Clone, Copy)]
pub enum LockType {
    Mutex,
    RwLock,
}

impl std::fmt::Display for LockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mutex => write!(f, "Mutex"),
            Self::RwLock => write!(f, "RwLock"),
        }
    }
}

/// Indicates how a lock was acquired.
#[derive(Debug, Clone, Copy)]
pub enum LockAcquisition {
    Read,
    Write,
}

impl std::fmt::Display for LockAcquisition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "Read"),
            Self::Write => write!(f, "Write"),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct LockInfoOwned {
    pub name: Option<String>,
    pub lock_type: LockType,
}
impl LockInfoOwned {
    pub fn as_lock_info(&self) -> LockInfo<'_> {
        LockInfo {
            name: self.name.as_deref(),
            lock_type: self.lock_type,
        }
    }
}

/// Contains metadata about a lock
#[derive(Debug, Clone)]
pub struct LockInfo<'a> {
    name: Option<&'a str>,
    lock_type: LockType,
}
impl<'a> LockInfo<'a> {
    /// get the lock's name
    pub fn name(&self) -> Option<&str> {
        self.name
    }

    /// get the lock's type
    pub fn lock_type(&self) -> LockType {
        self.lock_type
    }
}

#[derive(Debug, Clone)]
pub(super) struct LockCallbackInfo {
    pub lock_info_owned: LockInfoOwned,
    pub lock_callback_fn: Option<LockCallbackFn>,
}
impl LockCallbackInfo {
    pub fn new(
        lock_type: LockType,
        name: Option<String>,
        lock_callback_fn: Option<LockCallbackFn>,
    ) -> Self {
        Self {
            lock_info_owned: LockInfoOwned { name, lock_type },
            lock_callback_fn,
        }
    }
}

/// Represents an event (acquire/release) for a lock
#[derive(Debug, Clone)]
pub enum LockEvent<'a> {
    Acquire {
        info: LockInfo<'a>,
        acquired: LockAcquisition,
    },
    Release {
        info: LockInfo<'a>,
        acquired: LockAcquisition,
    },
}

/// A callback fn for receiving [LockEvent] event
/// each time a lock is acquired or released.
pub type LockCallbackFn = fn(lock_event: LockEvent);
