use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Ptr<T>(Rc<T>);
pub struct PtrMut<T>(Rc<RefCell<T>>);

pub struct Type {}

pub struct Object {
    ty: Ptr<Type>,
    properties: HashMap<Ptr<String>, Value>,
}

pub struct Code {}

pub struct Frame {}

pub struct Function {}

pub enum Value {
    Object(PtrMut<Object>),
    String(Ptr<String>),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Code(Ptr<Code>),
    Frame(PtrMut<Frame>),
    Function(Ptr<Function>),
}
