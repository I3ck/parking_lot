parking_lot
============

[![Build Status](https://travis-ci.org/Amanieu/parking_lot.svg?branch=master)](https://travis-ci.org/Amanieu/parking_lot) [![Build status](https://ci.appveyor.com/api/projects/status/wppcc32ttpud0a30/branch/master?svg=true)](https://ci.appveyor.com/project/Amanieu/parking-lot/branch/master) [![Crates.io](https://img.shields.io/crates/v/parking_lot.svg)](https://crates.io/crates/parking_lot)

[Documentation](https://amanieu.github.io/parking_lot/parking_lot/index.html)

This library provides implementations of `Mutex`, `RwLock`, `Condvar` and
`Once` that are smaller, faster and more flexible than those in the Rust
standard library. It also exposes a low-level API for creating your own
efficient synchronization primitives.

When tested on x86_64 Linux, `parking_lot::Mutex` was found to be 1.5x
faster than `std::sync::Mutex` when uncontended, and up to 3x faster when
contended from multiple threads. The numbers for `RwLock` vary depending on
the number of reader and writer threads, but are almost always faster than
the standard library `RwLock`, even up to 10x faster in some cases.

## Features

The primitives provided by this library have several advantages over those
in the Rust standard library:

1. `Mutex` and `Once` only require 1 byte of storage space, while `Condvar`
   and `RwLock` only require 1 word of storage space. On the other hand the
   standard library primitives require a dynamically allocated `Box` to hold
   OS-specific synchronization primitives. The small size of `Mutex` in
   particular encourages the use of fine-grained locks to increase
   parallelism.
2. Since they consist of just a single atomic variable, have constant
   initializers and don't need destructors, these primitives can be used as
    `static` global variables. The standard library primitives require
   dynamic initialization and thus need to be lazily initialized with
   `lazy_static!`.
3. Uncontended lock acquisition and release is done through fast inline
   paths which only require a single atomic operation.
4. Microcontention (a contended lock with a short critical section) is
   efficiently handled by spinning a few times while trying to acquire a
   lock.
5. The locks are adaptive and will suspend a thread after a few failed spin
   attempts. This makes the locks suitable for both long and short critical
   sections.
6. `Condvar` and `RwLock` work on Windows XP, unlike the standard library
   versions of those types.
7. `RwLock` takes advantage of hardware lock elision on processors that
   support it, which can lead to huge performance wins with many readers.

## The parking lot

To keep these primitives small, all thread queuing and suspending
functionality is offloaded to the *parking lot*. The idea behind this is
based on the Webkit [`WTF::ParkingLot`]
(https://webkit.org/blog/6161/locking-in-webkit/) class, which essentially
consists of a hash table mapping of lock addresses to queues of parked
(sleeping) threads. The Webkit parking lot was itself inspired by Linux
[futexes](http://man7.org/linux/man-pages/man2/futex.2.html), but it is more
powerful since it allows invoking callbacks while holding a queue lock.

*Parking* refers to suspending the thread while simultaneously enqueuing it
on a queue keyed by some address. *Unparking* refers to dequeuing a thread
from a queue keyed by some address and resuming it. The parking lot API
consists of just 4 functions:

```rust,ignore
unsafe fn park(key: usize,
               validate: &mut FnMut() -> bool,
               before_sleep: &mut FnMut(),
               timed_out: &mut FnMut(usize, UnparkResult),
               timeout: Option<Instant>)
               -> bool
```

This function performs the following steps:

1. Lock the queue associated with `key`.
2. Call `validate`, if it returns `false`, unlock the queue and return.
3. Add the current thread to the queue.
4. Unlock the queue.
5. Call `before_sleep`.
6. Sleep until we are unparked or `timeout` is reached.
7. If the park timed out, call `timed_out`.
8. Return `true` if we were unparked by another thread, `false` otherwise.

```rust,ignore
unsafe fn unpark_one(key: usize,
                     callback: &mut FnMut(UnparkResult))
                     -> UnparkResult
```

This function will unpark a single thread from the queue associated with
`key`. The `callback` function is invoked while holding the queue lock but
before the thread is unparked. The `UnparkResult` indicates whether the
queue was empty and, if not, whether there are still threads remaining in
the queue.

```rust,ignore
unsafe fn unpark_all(key: usize) -> usize
```

This function will unpark all threads in the queue associated with `key`. It
returns the number of threads that were unparked.

```rust,ignore
unsafe fn unpark_requeue(key_from: usize,
                         key_to: usize,
                         validate: &mut FnMut() -> RequeueOp,
                         callback: &mut FnMut(RequeueOp, usize))
                         -> usize
```

This function will remove all threads from the queue associated with
`key_from`, optionally unpark the first one and move the rest to the queue
associated with `key_to`. The `validate` function is invoked while holding
both queue locks and can choose whether to abort the operation and whether
to unpark one thread from the queue. The `callback` function is then called
with the result of `validate` and the number of threads that were in the
original queue.

## Building custom synchronization primitives

Building custom synchronization primitives is very simple since
`parking_lot` takes care of all the hard parts for you. The most common
case for a custom primitive would be to integrate a `Mutex` inside another
data type. Since a mutex only requires 2 bits, it can share space with other
data. For example, one could create an `ArcMutex` type that combines the
atomic reference count and the two mutex bits in the same atomic word.

## Nightly vs stable

There are a few restrictions when using this library on stable Rust:

- `Mutex` and `Once` will use 1 word of space instead of 1 byte.
- You will have to use `lazy_static!` to statically initialize `Mutex`,
  `Condvar` and `RwLock` types instead of `const fn`.
- `RwLock` will not be able to take advantage of hardware lock elision for
  readers, which improves performance when there are multiple readers.
- Slightly less efficient code may be generated for `compare_exchange`
  operations. This should not affect architectures like x86 though.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
parking_lot = "0.2"
```

and this to your crate root:

```rust
extern crate parking_lot;
```

To enable nightly-only features, add this to your `Cargo.toml` instead:

```toml
[dependencies]
parking_lot = {version = "0.2", features = ["nightly"]}
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
