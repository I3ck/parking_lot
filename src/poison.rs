// Copyright 2016 Amanieu d'Antras
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::thread;
pub use std::sync::{PoisonError, TryLockError, TryLockResult, LockResult};

pub struct Poison(pub bool);

impl Poison {
    #[inline]
    pub fn map_lock_result<T>(&self, val: T) -> LockResult<T> {
        if self.0 {
            Err(PoisonError::new(val))
        } else {
            Ok(val)
        }
    }
}

pub struct PoisonGuard {
    panicking: bool,
}

impl PoisonGuard {
    #[inline]
    pub fn new() -> PoisonGuard {
        PoisonGuard { panicking: thread::panicking() }
    }

    #[inline]
    pub fn done(&self) -> Poison {
        Poison(!self.panicking && thread::panicking())
    }
}
