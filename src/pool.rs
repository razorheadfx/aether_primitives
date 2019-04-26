use std::boxed::Box;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

/// Creates an object pool with the given initial size,
/// a ```maker``` function which creates new elements
/// and a ```resetter``` function which resets elements upon their return
/// to the pool.
///
/// This is useful for large buffers and other time expensive objects
/// under the assumption that there is minimal lock contention.
/// # Example
/// ```
/// use std::sync::mpsc;
/// use std::thread;
/// use aether_primitives::pool::{self,Pool};
///
/// let no_elems = 0usize;
/// let maker = Box::new(|| Vec::with_capacity(50));
/// let resetter = Box::new(|o: &mut Vec<u8>| o.clear());
///
/// let pool: Pool<Vec<u8>> = pool::make(no_elems, maker, resetter);
///
/// // clone a reference to the pool
/// let p = pool.clone();
/// assert_eq!(pool.len(), 0, "Pool should be empty");
/// {
///     let (tx, rx) = mpsc::channel();
///     let handle = thread::spawn(move || {
///         let elem = p.take_or_make();
///         tx.send(elem).unwrap();
///     });
///     let _elem = rx.recv();
///     assert_eq!(pool.len(), 0, "Pool should be empty");
///     assert_eq!(pool.cap(), 1, "Pool should own 1 element");
///     handle.join().unwrap();
/// } // drop the guard so the element is returned to the pool
///
/// assert_eq!(pool.len(), 1);
/// assert_eq!(pool.cap(), 1);
/// ```
pub fn make<T>(
    initial_len: usize,
    maker: Box<dyn Fn() -> T + Send>,
    resetter: Box<dyn Fn(&mut T) + Send>,
) -> Pool<T> {
    let elems = (0..initial_len)
        .map(|_| maker())
        .map(|mut e| {
            resetter(&mut e);
            e
        })
        .collect::<Vec<_>>();

    let cap = elems.len();
    let pool = PoolInner {
        elems,
        maker,
        resetter,
        cap,
    };

    let inner = Mutex::new(pool);
    let inner = Arc::new(inner);
    Pool { inner }
}

pub struct Pool<T> {
    inner: Arc<Mutex<PoolInner<T>>>,
}

impl<T> Pool<T> {
    /// Either returns a guard which derefs into a ```T``` or nothing, depending
    /// on whether the pool is empty or not.
    /// Once the guard is dropped it will return the element to the pool.
    ///
    /// This corresponds to a bounded usage as no new ```T``` will be created after
    /// the pool is initialised.
    pub fn take(&self) -> Option<Elem<T>> {
        let mut i = match self.inner.lock() {
            Ok(i) => i,
            Err(_e) => return None,
        };

        if i.elems.is_empty() {
            None
        } else {
            let val = i.elems.pop().expect("Pool was empty when it should not be");
            let e = Elem {
                pool: Arc::clone(&self.inner),
                val: ManuallyDrop::new(val),
            };
            Some(e)
        }
    }

    /// Clones a reference to the pool
    pub fn clone(&self) -> Pool<T> {
        Pool {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Takes an element from the pool if it is not empty
    /// otherwise creates a new element using the ```maker``` associated
    /// with this pool, increasing the capacity of the pool by 1.
    ///
    /// This corresponds to an unbounded use case as the pool will grow to
    /// meet demand.
    /// # Panics
    /// Panics if the underlying mutex was poisoned by another thread panicking
    /// while holding the lock.
    pub fn take_or_make(&self) -> Elem<T> {
        let val = {
            let mut i = self.inner.lock().expect("Mutex was poisoned");
            if i.elems.is_empty() {
                // call the maker function
                let new_elem = (i.maker)();
                i.cap += 1;
                new_elem
            } else {
                i.elems.pop().expect("Pool should not be empty")
            }
        }; // unlock the mutex

        Elem {
            pool: Arc::clone(&self.inner),
            val: ManuallyDrop::new(val),
        }
    }

    /// Returns the number of elements currently held in this pool.
    /// # Panics
    /// Panics if the underlying mutex was poisoned by another thread panicking
    /// while holding the lock.
    pub fn len(&self) -> usize {
        self.inner.lock().expect("Mutex was poisoned").elems.len()
    }

    /// Checks whether the pool is empty
    /// # Panics
    /// Panics if the underlying mutex was poisoned by another thread panicking
    /// while holding the lock.
    pub fn is_emtpy(&self) -> bool {
        self.inner
            .lock()
            .expect("Mutex was poisoned")
            .elems
            .is_empty()
    }

    /// Returns the number of elements associated with this pool.
    /// # Panics
    /// Panics if the underlying mutex was poisoned by another thread panicking
    /// while holding the lock.
    pub fn cap(&self) -> usize {
        self.inner.lock().expect("Mutex was poisoned").cap
    }
}

struct PoolInner<T> {
    /// A vec holding all currently checked-in elements
    elems: Vec<T>,
    /// A function used to create elements either on making the pool or when calling [take_or_make()](Pool::take_or_make).
    ///
    maker: Box<Fn() -> T + Send>,
    /// A function used to reset elements upon their return to the pool
    resetter: Box<Fn(&mut T) + Send>,
    /// Total number of elements owned by this Pool.
    /// This is on contrast to len, which is the number of elements currently available
    cap: usize,
}

impl<T> PoolInner<T> {
    /// Resets and returns the given element to the pool
    pub fn give_back(&mut self, mut val: T) {
        (self.resetter)(&mut val);
        self.elems.push(val)
    }
}

/// A guard representing an element associated with a pool.
///
/// Derefs into ```T```.
///
/// Once this guard is dropped it will be reset using the pool's ```resetter```
/// and returned to the pool
pub struct Elem<T> {
    pool: Arc<Mutex<PoolInner<T>>>,
    val: ManuallyDrop<T>,
}

impl<T> Drop for Elem<T> {
    fn drop(&mut self) {
        match self.pool.lock() {
            Ok(mut p) => {
                // remove the element from the struct
                let v = unsafe { ManuallyDrop::take(&mut self.val) };
                p.give_back(v);
            }
            Err(_e) => {
                println!("Mutex was poisoned; Dropping Element");
                unsafe { ManuallyDrop::drop(&mut self.val) }
            }
        }
    }
}

impl<T> Deref for Elem<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.val
    }
}

impl<T> DerefMut for Elem<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.val
    }
}

#[cfg(test)]
mod test {
    use crate::pool;
    use crate::pool::{Elem, Pool};

    #[test]
    pub fn taking() {
        let cap = 1usize;
        let maker = || Vec::with_capacity(50);
        let resetter = |o: &mut Vec<u8>| o.clear();

        let pool: Pool<Vec<u8>> = pool::make(cap, Box::new(maker), Box::new(resetter));
        assert_eq!(pool.len(), 1usize);
        assert_eq!(pool.cap(), 1usize);

        {
            let c1: Option<Elem<Vec<u8>>> = pool.take();
            assert!(c1.is_some(), "First time checkout failed");
            assert_eq!(pool.len(), 0usize);
            assert_eq!(pool.cap(), 1usize);
        }
        assert_eq!(pool.len(), 1usize);
        assert_eq!(pool.cap(), 1usize);

        // pool should be empty now
        // should panic exactly here
        {
            let c1: Option<Elem<Vec<u8>>> = pool.take();
            assert!(
                c1.is_some(),
                "Second checkout failed, when it should have succeeded"
            );
            let c2: Option<Elem<Vec<u8>>> = pool.take();
            assert!(
                c2.is_none(),
                "Third checkout succeeded when it should have failed"
            )
        }
        assert_eq!(pool.len(), 1usize);
        assert_eq!(pool.cap(), 1usize);
    }

    #[test]
    fn resetting() {
        let maker = || Vec::with_capacity(50);
        let resetter = |o: &mut Vec<u8>| o.clear();
        let pool: Pool<Vec<u8>> = pool::make(1, Box::new(maker), Box::new(resetter));

        {
            let mut first_elem = pool.take().unwrap();
            (0..50).for_each(|x| first_elem.push(x as u8));
            assert_eq!(first_elem.len(), 50, "Vector should contain elements now");
        }
    }

    #[test]
    pub fn taking_or_making() {
        let cap = 0usize;
        let maker = || Vec::with_capacity(50);
        let resetter = |o: &mut Vec<u8>| o.clear();

        let pool: Pool<Vec<u8>> = pool::make(cap, Box::new(maker), Box::new(resetter));
        {
            let _e1: Elem<Vec<u8>> = pool.take_or_make();
            // pool should be empty now
            assert_eq!(pool.len(), 0usize);
            assert_eq!(pool.cap(), 1usize);
            let _e2: Elem<Vec<u8>> = pool.take_or_make();
            assert_eq!(pool.len(), 0usize);
            assert_eq!(pool.cap(), 2usize);
        } // drops both elem guards, should
        assert_eq!(pool.len(), 2usize);
        assert_eq!(pool.cap(), 2usize);
    }
}
