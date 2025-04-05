use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use async_ringbuf::{AsyncHeapCons, AsyncHeapProd};

/// Wrapper for the producer side of the ringbuffer to provide a custom Debug impl
pub struct ProdWrap(AsyncHeapProd<u8>);

impl ProdWrap {
    pub(super) fn new(inner: AsyncHeapProd<u8>) -> Self {
        Self(inner)
    }
}

impl Deref for ProdWrap {
    type Target = AsyncHeapProd<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProdWrap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for ProdWrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ProdWrap").finish()
    }
}

/// Wrapper for the consumer side of the ringbuffer to provide a custom Debug impl
pub struct ConsWrap(AsyncHeapCons<u8>);

impl ConsWrap {
    pub(super) fn new(inner: AsyncHeapCons<u8>) -> Self {
        Self(inner)
    }
}

impl Deref for ConsWrap {
    type Target = AsyncHeapCons<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ConsWrap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for ConsWrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ConsWrap").finish()
    }
}
