use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};
use spin::{Mutex, MutexGuard};

use crate::arch::irq::Irq;

pub struct MutexNoIrq<T: ?Sized> {
    lock: Mutex<T>,
}

/// Irq Status Struct.
/// This structure contains the status of the current IRQ
/// And it will restore irq status after dropping.
struct IrqStatus {
    irq_enabled: bool,
}

/// Restore the IRQ status when dropping
impl Drop for IrqStatus {
    fn drop(&mut self) {
        if self.irq_enabled {
            Irq::enable_interrupt();
        }
    }
}

/// Implement Sync for MutexNoIrq
/// # Safety
/// The MutexNoIrq ensures that interrupts are disabled while the lock is held (or being acquired),
/// preventing deadlocks or reentrancy issues with interrupt handlers attempting to acquire the same lock.
/// This makes it safe to be shared across threads in an environment where IRQs interact with locks.
unsafe impl<T: ?Sized + Send> Sync for MutexNoIrq<T> {}

/// Implement Send for MutexNoIrq
/// # Safety
/// If T is Send, and the lock mechanism is sound (including IRQ management),
/// the MutexNoIrq as a whole can be sent to another thread.
unsafe impl<T: ?Sized + Send> Send for MutexNoIrq<T> {}

impl<T> MutexNoIrq<T> {
    pub const fn new(data: T) -> MutexNoIrq<T> {
        MutexNoIrq {
            lock: Mutex::new(data),
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.lock.into_inner()
    }
}

impl<T: ?Sized> MutexNoIrq<T> {
    /// Attempts to acquire the lock. If successful, interrupts will have been disabled
    /// and will be restored when the returned guard is dropped.
    /// If the lock cannot be acquired, interrupts are restored to their previous state,
    /// and None is returned.
    #[inline]
    pub fn try_lock(&self) -> Option<MutexNoIrqGuard<T>> {
        let original_irq_status = IrqStatus {
            irq_enabled: Irq::interrupt_enabled(),
        };

        Irq::disable_interrupt();

        // If lock is not acquired. IRQs were disabled by us.
        // original_irq_status was not moved and will be dropped now,
        // automatically restoring the original IRQ state.
        self.lock.try_lock().map(|guard| MutexNoIrqGuard {
            guard,
            _irq_status: original_irq_status,
        })
    }

    /// Acquires the lock, spinning if necessary.
    /// Interrupts are disabled before attempting to acquire the lock and will
    /// remain disabled while spinning. They are restored when the returned guard is dropped.
    #[inline]
    pub fn lock(&self) -> MutexNoIrqGuard<T> {
        let original_irq_status_keeper = IrqStatus {
            irq_enabled: Irq::interrupt_enabled(),
        };

        Irq::disable_interrupt();

        // Spin until the underlying lock is acquired.
        let acquired_guard = loop {
            if let Some(guard) = self.lock.try_lock() {
                break guard;
            }
            spin_loop();
        };

        MutexNoIrqGuard {
            guard: acquired_guard,
            _irq_status: original_irq_status_keeper,
        }
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.lock.is_locked()
    }

    /// Forcibly unlocks the mutex.
    ///
    /// # Safety
    ///
    /// This is incredibly unsafe, as it can lead to data corruption and deadlocks.
    /// It does not restore interrupt state; that is tied to the `MutexNoIrqGuard`.
    /// This method should only be used in recovery code where the lock state is known
    /// to be inconsistent.
    pub unsafe fn force_unlock(&self) {
        unsafe { self.lock.force_unlock() }
    }
}

/// The Mutex Guard that also manages IRQ state restoration.
pub struct MutexNoIrqGuard<'a, T: ?Sized + 'a> {
    guard: MutexGuard<'a, T>,
    _irq_status: IrqStatus,
}

impl<T: ?Sized> Deref for MutexNoIrqGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &(self.guard)
    }
}

impl<T: ?Sized> DerefMut for MutexNoIrqGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut (self.guard)
    }
}
