use core::{
    cell::UnsafeCell,
    fmt,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};

/// A thread-safe cell that can be initialized exactly once.
///
/// This is a more efficient alternative to using `static mut` with unsafe code.
/// It uses MaybeUninit to avoid Option overhead and uses AtomicBool to track
/// initialization.
pub struct OnceCell<T> {
    initialized: AtomicBool,
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send> Send for OnceCell<T> {}
unsafe impl<T: Send + Sync> Sync for OnceCell<T> {}

impl<T> OnceCell<T> {
    /// Creates a new uninitialized static cell.
    pub const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Initializes the cell with a value.
    ///
    /// # Panics
    ///
    /// Panics if the cell is already initialized.
    pub fn init(&self, value: T) {
        // Ensure the cell hasn't been initialized already
        if self.initialized.swap(true, Ordering::AcqRel) {
            panic!("StaticCell already initialized");
        }

        // SAFETY: We've ensured this is only called once via the atomic flag
        unsafe {
            (*self.value.get()).write(value);
        }
    }

    /// Gets a reference to the value.
    ///
    /// # Panics
    ///
    /// Panics if the cell has not been initialized.
    #[inline]
    pub fn get(&self) -> &T {
        if !self.initialized.load(Ordering::Acquire) {
            panic!("StaticCell not initialized");
        }

        // SAFETY: We've verified the cell is initialized
        unsafe { &*(*self.value.get()).as_ptr() }
    }

    /// Gets a mutable reference to the value if initialized.
    ///
    /// Requires `&mut self` (exclusive access), making it safe for concurrency.
    /// Returns `None` if the cell is not initialized.
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        // Since we have &mut self, no other thread can access. Relaxed is fine.
        // Use .get_mut() on the AtomicBool for direct mutable access to the flag.
        if *self.initialized.get_mut() {
            // SAFETY: We have exclusive access (&mut self) and it's initialized.
            Some(unsafe { (*self.value.get()).assume_init_mut() })
        } else {
            None
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for OnceCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.initialized.load(Ordering::Acquire) {
            write!(
                f,
                "StaticCell {{ initialized: true, value: {:?} }}",
                self.get()
            )
        } else {
            write!(f, "StaticCell {{ initialized: false }}")
        }
    }
}

impl<T> Default for OnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for OnceCell<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for OnceCell<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // Panic if not initialized. Safe because get_mut requires &mut self.
        self.get_mut()
            .expect("OnceCell::deref_mut accessed before initialization")
    }
}

impl<T> Drop for OnceCell<T> {
    fn drop(&mut self) {
        if self.initialized.load(Ordering::Relaxed) {
            unsafe {
                ptr::drop_in_place(self.value.get());
            }
        }
    }
}
