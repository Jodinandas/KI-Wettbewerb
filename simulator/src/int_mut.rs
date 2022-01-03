use std::{
    ptr,
    sync::{Arc, Mutex, MutexGuard, Weak},
};

/// This struct implements the interior mutability pattern and
/// is basically only used to store data and make the access
/// to it easier
#[derive(Debug)]
pub struct IntMut<T> {
    /// data
    data: Arc<Mutex<T>>,
}
impl<T> IntMut<T> {
    /// creates a new IntMut
    pub fn new(data: T) -> IntMut<T> {
        IntMut {
            data: Arc::new(Mutex::new(data)),
        }
    }
    /// returns a [MutexGuard] to the data
    pub fn get(&self) -> MutexGuard<T> {
        (*self.data).lock().unwrap()
    }
    /// returns a [WeakIntMut] by calling downgrade on the internal Arc
    pub fn downgrade(&self) -> WeakIntMut<T> {
        WeakIntMut {
            data: Arc::downgrade(&self.data),
        }
    }

}
impl<T: Clone> IntMut<T> {
    /// deep copy of the IntMut
    pub fn deep_copy(&self) -> IntMut<T> {
        let new_data = (*self.get()).clone();
        IntMut::new(new_data)
    }
}
impl<T> Clone for IntMut<T> {
    /// BE CAREFUL: SHALLOW COPY
    fn clone(&self) -> Self {
        Self { data: Arc::clone(&self.data) }
    }
}

//impl<T> Deref for IntMut<T> {
//    type Target = T;
//
//    fn deref(&self) -> &'_ Self::Target {
//        &*self.get()
//    }
//}
//
//impl<T> DerefMut for IntMut<T> {
//    fn deref_mut(&mut self) -> &mut Self::Target {
//        &mut *self.get()
//    }
//}
//impl<T> Deref for WeakIntMut<T> {
//    type Target = T;
//
//    fn deref(&self) -> &Self::Target {
//        &*self.upgrade().get()
//    }
//}
//
//impl<T> DerefMut for WeakIntMut<T> {
//    fn deref_mut(&mut self) -> &mut Self::Target {
//        &mut *self.upgrade().get()
//    }
//}

/// Like [IntMut] but storing a weak reference
#[derive(Debug, Clone)]
pub struct WeakIntMut<T> {
    /// data
    data: Weak<Mutex<T>>,
}
impl<T> WeakIntMut<T> {
    /// Upgrades the reference to a strong referenced IntMut
    ///
    /// panics if the data the [WeakIntMut] references doesn't exist
    /// anymore
    pub fn upgrade(&self) -> IntMut<T> {
        IntMut {
            data: self.data.upgrade().unwrap(),
        }
    }
    /// Tries to upgrade the reference to an IntMut
    ///
    /// Returns Some(upgraded) or [None] if the reference doesn't exist
    /// anymore
    pub fn try_upgrade(&self) -> Option<IntMut<T>> {
        match self.data.upgrade() {
            Some(upgraded_data) => Some(IntMut {
                data: upgraded_data,
            }),
            None => None,
        }
    }

    /// calls as_ptr on the inner data and returns the result
    pub fn data_as_ptr(&self) -> *const Mutex<T> {
        self.data.as_ptr()
    }
}

impl<T> PartialEq<WeakIntMut<T>> for IntMut<T> {
    fn eq(&self, other: &WeakIntMut<T>) -> bool {
        ptr::eq(other.data_as_ptr(), &*self.data)
    }
}
impl<T> PartialEq<IntMut<T>> for WeakIntMut<T> {
    fn eq(&self, other: &IntMut<T>) -> bool {
        ptr::eq(self.data_as_ptr(), &*other.data)
    }
}
impl<T> PartialEq<WeakIntMut<T>> for WeakIntMut<T> {
    fn eq(&self, other: &WeakIntMut<T>) -> bool {
        ptr::eq(other.data_as_ptr(), self.data_as_ptr())
    }
}
impl<T> PartialEq<IntMut<T>> for IntMut<T> {
    fn eq(&self, other: &IntMut<T>) -> bool {
        ptr::eq(&*other.data, &*self.data)
    }
}

mod tests {

    #[test]
    fn partial_eq() {
        use super::IntMut;
        let a = IntMut::new(5);
        let b = IntMut::new(6);
        let c = IntMut::new(5);
        let d = IntMut::new(6);
        let wa = a.downgrade();
        let wb = b.downgrade();
        let wc = c.downgrade();
        let wd = d.downgrade();
        // weak and strong IntMuts must be comparable
        assert_eq!(a, wa);
        assert_eq!(b, wb);
        assert_eq!(c, wc);
        assert_eq!(d, wd);
        // IntMuts pointing to equal values but different data should not be equal
        assert_ne!(a, c);
        assert_ne!(b, d);
        // WeakIntMuts pointing to equal values but different data should not be equal
        assert_ne!(wa, wc);
        assert_ne!(wb, wd);
        // WeakIntMuts pointing to the same object should be equal
        assert_eq!(wa, a.downgrade());
        assert_eq!(wb, b.downgrade());
        // WeakIntMuts and IntMuts pointing to the same object should be equal
        assert_eq!(a, wa);
        assert_eq!(b, wb);
        // WeakIntMuts and IntMuts pointing to different objects with the same values should not be equal
        assert_ne!(a, wb);
        assert_ne!(wa, b);
    }
    //#[test]
    //fn deref_intmut() {
    //    use super::IntMut;

    //    let a = IntMut::new(4);
    //    assert_eq!(*a, 4);
    //}

    //#[test]
    //fn derefmut_intmut() {
    //    use super::IntMut;

    //    let a = IntMut::new(4);
    //    *a = 5;
    //    assert_eq!(*a, 5);
    //}

    //#[test]
    //fn deref_weakintmut() {
    //    use super::{IntMut};

    //    let a = IntMut::new(4);
    //    let wa = a.downgrade();
    //    assert_eq!(*wa, 4);
    //}

    //#[test]
    //fn derefmut_weakintmut() {
    //    use super::{IntMut};

    //    let a = IntMut::new(4);
    //    let wa = a.downgrade();
    //    *wa = 5;
    //    assert_eq!(*wa, 5)
    //}
}
