use super::{inner::MemoInner, ArcMemo};
use crate::{
    owner::StoredValue,
    signal::guards::{Mapped, Plain, ReadGuard},
    traits::{DefinedAt, Dispose, ReadUntracked, Track},
    unwrap_signal,
};
use std::{fmt::Debug, hash::Hash, panic::Location};

pub struct Memo<T> {
    #[cfg(debug_assertions)]
    defined_at: &'static Location<'static>,
    inner: StoredValue<ArcMemo<T>>,
}

impl<T: 'static> Dispose for Memo<T> {
    fn dispose(self) {
        self.inner.dispose()
    }
}

impl<T: Send + Sync + 'static> From<ArcMemo<T>> for Memo<T> {
    #[track_caller]
    fn from(value: ArcMemo<T>) -> Self {
        Self {
            #[cfg(debug_assertions)]
            defined_at: Location::caller(),
            inner: StoredValue::new(value),
        }
    }
}

impl<T: Send + Sync + 'static> Memo<T> {
    #[track_caller]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all,)
    )]
    pub fn new(fun: impl Fn(Option<&T>) -> T + Send + Sync + 'static) -> Self
    where
        T: PartialEq,
    {
        Self {
            #[cfg(debug_assertions)]
            defined_at: Location::caller(),
            inner: StoredValue::new(ArcMemo::new(fun)),
        }
    }

    #[track_caller]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip_all,)
    )]
    pub fn new_with_compare(
        fun: impl Fn(Option<&T>) -> T + Send + Sync + 'static,
        changed: fn(Option<&T>, Option<&T>) -> bool,
    ) -> Self
    where
        T: PartialEq,
    {
        Self {
            #[cfg(debug_assertions)]
            defined_at: Location::caller(),
            inner: StoredValue::new(ArcMemo::new_with_compare(fun, changed)),
        }
    }

    #[track_caller]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip_all,)
    )]
    pub fn new_owning(
        fun: impl Fn(Option<T>) -> (T, bool) + Send + Sync + 'static,
    ) -> Self
    where
        T: PartialEq,
    {
        Self {
            #[cfg(debug_assertions)]
            defined_at: Location::caller(),
            inner: StoredValue::new(ArcMemo::new_owning(fun)),
        }
    }
}

impl<T> Copy for Memo<T> {}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Debug for Memo<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Memo")
            .field("type", &std::any::type_name::<T>())
            .field("store", &self.inner)
            .finish()
    }
}

impl<T> PartialEq for Memo<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for Memo<T> {}

impl<T> Hash for Memo<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<T> DefinedAt for Memo<T> {
    fn defined_at(&self) -> Option<&'static Location<'static>> {
        #[cfg(debug_assertions)]
        {
            Some(self.defined_at)
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    }
}

impl<T: Send + Sync + 'static> Track for Memo<T> {
    #[track_caller]
    fn track(&self) {
        if let Some(inner) = self.inner.get() {
            inner.track();
        }
    }
}

impl<T: Send + Sync + 'static> ReadUntracked for Memo<T> {
    type Value = ReadGuard<T, Mapped<Plain<MemoInner<T>>, T>>;

    fn try_read_untracked(&self) -> Option<Self::Value> {
        self.inner.get().map(|inner| inner.read_untracked())
    }
}

impl<T: Send + Sync + 'static> From<Memo<T>> for ArcMemo<T> {
    #[track_caller]
    fn from(value: Memo<T>) -> Self {
        value.inner.get().unwrap_or_else(unwrap_signal!(value))
    }
}
