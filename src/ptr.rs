use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

#[derive(Hash, PartialEq, Eq)]
pub struct Ptr<T>(Rc<T>);

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Ptr<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }

    pub fn value(&self) -> &T {
        &self.0
    }

    pub fn id(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
}

pub struct PtrMut<T>(Rc<RefCell<T>>);

impl<T> Clone for PtrMut<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PtrMut<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn value(&self) -> Ref<T> {
        self.0.borrow()
    }

    pub fn value_mut(&self) -> RefMut<T> {
        self.0.borrow_mut()
    }

    pub fn id(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
}
