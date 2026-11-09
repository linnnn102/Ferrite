use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// A lock that makes a value safe to share: only one user at a time.
pub struct Spinlock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: the lock guarantees only one `&mut T` exists at a time, so sharing a
// `Spinlock<T>` between threads/interrupts is fine as long as `T` can be sent.
unsafe impl<T: Send> Sync for Spinlock<T> {}

impl<T> Spinlock<T> {
    /// Creates an unlocked spinlock holding `data`.
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Waits until the lock is free, takes it, and returns a guard.
    /// The lock is released when the guard is dropped.
    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        SpinlockGuard { lock: self }
    }
}

/// Proof that you hold the lock. Gives access to the data; unlocks when dropped.
pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
}

impl<T> Deref for SpinlockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: holding the guard means we hold the lock.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: holding the guard means we hold the lock.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinlockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}
