//! A mutex implementation based on the rp2040 hardware spinlock.
//!
//! # Example
//!
//! Fully working code can be found in `examples/`.
//!
//! ```no_run
//! use rp_spinlockmutex::SpinlockMutex;
//! static MUTEX: SpinlockMutex<7, i32> = SpinlockMutex::new(0);
//!
//! run_on_core1(|| {
//!     for _ in 0..10 {
//!         *MUTEX.lock() += 1;
//!     }
//! });
//!
//! for _ in 0..10 {
//!     *MUTEX.lock() += 1;
//! }
//!
//! assert_eq!(*mutex.lock(), 20);
//! ```
#![no_std]

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use rp2040_hal::sio::{Spinlock, SpinlockValid};

/// A mutex implementation based on the rp2040 hardware spinlock.
///
/// The rp2040 provides 32 hardware spinlocks. The lock number (0 to 31)
/// is specified via a compile-time constant, e.g. `SpinlockMutex<7, _>`
/// uses Spinlock 7. (**Note:** `rp2040_hal` uses Spinlock 31 in their
/// `critical-section` implementation.)
///
/// These hardware spinlocks are global, so e.g. if you try to lock a
/// `SpinlockMutex<7, _>` then any other part of your application using
/// `SpinlockMutex<7, _>` or [`rp2040_hal::sio::Spinlock<7>`][`Spinlock`]
/// will contend for the same lock.
///
/// If both cores try to claim the lock on the same clock cycle,
/// core 0 will acquire the lock, which may lead to lock starvation.
///
/// # Example
///
/// Fully working code can be found in `examples/`.
///
/// ```no_run
/// use rp_spinlockmutex::SpinlockMutex;
/// static MUTEX: SpinlockMutex<7, i32> = SpinlockMutex::new(0);
///
/// run_on_core1(|| {
///     for _ in 0..10 {
///         *MUTEX.lock() += 1;
///     }
/// });
///
/// for _ in 0..10 {
///     *MUTEX.lock() += 1;
/// }
///
/// assert_eq!(*MUTEX.lock(), 20);
/// ```
pub struct SpinlockMutex<const N: usize, T: ?Sized>
where
    Spinlock<N>: SpinlockValid,
{
    data: UnsafeCell<T>,
}

unsafe impl<const N: usize, T: ?Sized + Send> Send for SpinlockMutex<N, T> where Spinlock<N>: SpinlockValid {}
unsafe impl<const N: usize, T: ?Sized + Send> Sync for SpinlockMutex<N, T> where Spinlock<N>: SpinlockValid {}

impl<const N: usize, T> SpinlockMutex<N, T>
where
    Spinlock<N>: SpinlockValid,
{
    /// Creates a new hardware based spinlock mutex in an unlocked state ready for use.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rp_spinlockmutex::SpinlockMutex;
    /// let mutex: SpinlockMutex<7, i32> = SpinlockMutex::new(42);
    /// ```
    #[inline]
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

impl<const N: usize, T: ?Sized> SpinlockMutex<N, T>
where
    Spinlock<N>: SpinlockValid,
{
    /// Acquires the mutex lock, blocking the current thread until the lock is available.
    ///
    /// # Deadlock
    ///
    /// Repeatedly calling while holding the lock will cause a deadlock.
    /// (**Note:** This also applies to interrupts as these are not deactivated.)
    ///
    /// ```no_run
    /// use rp_spinlockmutex::SpinlockMutex;
    /// let mutex: SpinlockMutex<7, i32> = SpinlockMutex::new(42);
    ///
    /// let guard_1 = mutex.lock();
    /// let guard_2 = mutex.lock(); // ❌ deadlock ❌
    /// ```
    #[inline]
    pub fn lock(&self) -> SpinlockMutexGuard<N, T> {
        SpinlockMutexGuard {
            _lock: Spinlock::<N>::claim(),
            data: self.data.get(),
        }
    }

    pub fn try_lock(&self) -> Option<SpinlockMutexGuard<N, T>> {
        Spinlock::<N>::try_claim().map(|lock| SpinlockMutexGuard {
            _lock: lock,
            data: self.data.get(),
        })
    }

    #[inline]
    pub fn unlock(guard: SpinlockMutexGuard<N, T>) {
        core::mem::drop(guard);
    }
}

/// A SpinlockMutexGuard allows the holder to access the protected data of a mutex.
/// If this guard is dropped, the mutex will be unlocked automatically. The lock can
/// also be lifted manually with [`SpinlockMutex::unlock`].
///
///
#[must_use = "if unused the SpinlockMutex will immediately unlock"]
pub struct SpinlockMutexGuard<const N: usize, T: ?Sized>
where
    Spinlock<N>: SpinlockValid,
{
    _lock: Spinlock<N>,
    data: *mut T,
}

unsafe impl<const N: usize, T: ?Sized + Send> Send for SpinlockMutexGuard<N, T> where Spinlock<N>: SpinlockValid {}
unsafe impl<const N: usize, T: ?Sized + Sync> Sync for SpinlockMutexGuard<N, T> where Spinlock<N>: SpinlockValid {}

impl<const N: usize, T: ?Sized> Deref for SpinlockMutexGuard<N, T>
where
    Spinlock<N>: SpinlockValid,
{
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: There can only ever be one instance of a mutex guard at the same time.
        //         Therefore it's safe to hand out borrows.
        unsafe { &*self.data }
    }
}

impl<const N: usize, T: ?Sized> DerefMut for SpinlockMutexGuard<N, T>
where
    Spinlock<N>: SpinlockValid,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: There can only ever be one instance of a mutex guard at the same time.
        //         Therefore it's safe to hand out borrows.
        unsafe { &mut *self.data }
    }
}
