# rp-spinlockmutex
![version](https://img.shields.io/crates/v/rp-spinlockmutex?style=flat-square)
![license](https://img.shields.io/crates/l/rp-spinlockmutex?style=flat-square)

A mutex implementation based on the rp2040 hardware spinlock.

## Example

Fully working code can be found in ``examples/``.

```rust
use rp_spinlockmutex::SpinlockMutex;
static MUTEX: SpinlockMutex<7, i32> = SpinlockMutex::new(0);

run_on_core1(|| {
    for _ in 0..10 {
        *MUTEX.lock() += 1;
    }
});

for _ in 0..10 {
    *MUTEX.lock() += 1;
}

assert_eq!(*mutex.lock(), 20);
```

## License
Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/license/mit/)

at your option.
