use crate::{
    get_native_prop,
    ptr::{Ptr, PtrMut},
    value::{ArgPattern, Type, Value},
    Interpreter,
};

pub struct BuiltinTypes {
    pub string: Ptr<Type>,
    pub float: Ptr<Type>,
    pub list: Ptr<Type>,
    pub integer: Ptr<Type>,
    pub bool: Ptr<Type>,
    pub function: Ptr<Type>,
    pub frame: Ptr<Type>,
    pub native: Ptr<Type>,
    pub code: Ptr<Type>,
    pub object: Ptr<Type>,
    pub null: Ptr<Type>,
    pub ty: Ptr<Type>,
}

impl Default for BuiltinTypes {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltinTypes {
    pub fn new() -> Self {
        let object_ty = object_ty();

        Self {
            object: object_ty.clone(),
            string: string_ty(object_ty.clone()),
            list: list_ty(object_ty.clone()),
            float: float_ty(object_ty.clone()),
            bool: bool_ty(object_ty.clone()),
            integer: integer_ty(object_ty.clone()),
            frame: frame_ty(object_ty.clone()),
            native: native_ty(object_ty.clone()),
            function: function_ty(object_ty.clone()),
            code: code_ty(object_ty.clone()),
            null: null_ty(object_ty.clone()),
            ty: ty_ty(object_ty),
        }
    }
}

fn string_ty(base: Ptr<Type>) -> Ptr<Type> {
    let mut ty = Type::new(Ptr::new("str".to_string()), base);

    ty.display = Some(Value::Native(
        |_, args| args[0].clone(),
        ArgPattern::Exact(1),
    ));

    ty.add = Some(Value::Native(
        |i, args| {
            Value::String(Ptr::new(format!(
                "{}{}",
                args[0].string(i).value(),
                args[1].string(i).value()
            )))
        },
        ArgPattern::Exact(2),
    ));

    Ptr::new(ty)
}

fn list_ty(base: Ptr<Type>) -> Ptr<Type> {
    let mut ty = Type::new(Ptr::new("list".to_string()), base);

    ty.display = Some(Value::Native(
        |interp, args| {
            let mut string = "[".to_string();

            let list = args[0].list(interp).value().to_vec();

            for i in 0..list.len() {
                if i > 0 {
                    string.push_str(", ")
                }

                let display = get_native_prop!(interp, list[i], display);
                string.push_str(
                    interp
                        .call_with_return(display, &list[i..i + 1])
                        .string(interp)
                        .value(),
                );
            }

            string.push(']');

            Value::String(Ptr::new(string))
        },
        ArgPattern::Exact(1),
    ));

    ty.get_subscript = Some(Value::Native(
        |i, args| {
            args[0]
                .list(i)
                .value()
                .get(args[1].as_int(i) as usize)
                .expect("index out of bounds")
                .clone()
        },
        ArgPattern::Exact(2),
    ));

    ty.set_subscript = Some(Value::Native(
        |i, args| {
            *args[0]
                .list(i)
                .value_mut()
                .get_mut(args[1].as_int(i) as usize)
                .expect("index out of bounds") = args[2].clone();

            Value::Null
        },
        ArgPattern::Exact(3),
    ));

    ty.add = Some(Value::Native(
        |i, args| {
            let mut new_list = args[0].list(i).value().clone();
            new_list.extend_from_slice(&*args[1].list(i).value());
            Value::List(PtrMut::new(new_list))
        },
        ArgPattern::Exact(2),
    ));

    ty.properties.insert(
        Ptr::new("push".to_string()),
        Value::Native(
            |i, args| {
                args[0].list(i).value_mut().push(args[1].clone());
                Value::Null
            },
            ArgPattern::Exact(2),
        ),
    );

    Ptr::new(ty)
}

fn integer_ty(base: Ptr<Type>) -> Ptr<Type> {
    let mut ty = Type::new(Ptr::new("int".to_string()), base);

    ty.display = Some(Value::Native(
        |i, args| Value::String(Ptr::new(format!("{}", args[0].int(i)))),
        ArgPattern::Exact(1),
    ));

    ty.add = Some(Value::Native(
        |i, args| Value::Integer(args[0].int(i) + args[1].as_int(i)),
        ArgPattern::Exact(2),
    ));

    ty.subtract = Some(Value::Native(
        |i, args| Value::Integer(args[0].int(i) - args[1].as_int(i)),
        ArgPattern::Exact(2),
    ));

    ty.divide = Some(Value::Native(
        |i, args| Value::Integer(args[0].int(i) / args[1].as_int(i)),
        ArgPattern::Exact(2),
    ));

    ty.multiply = Some(Value::Native(
        |i, args| Value::Integer(args[0].int(i) * args[1].as_int(i)),
        ArgPattern::Exact(2),
    ));

    ty.less_than = Some(Value::Native(
        |i, args| Value::Bool(args[0].int(i) < args[1].as_int(i)),
        ArgPattern::Exact(2),
    ));

    ty.greater_than = Some(Value::Native(
        |i, args| Value::Bool(args[0].int(i) > args[1].as_int(i)),
        ArgPattern::Exact(2),
    ));

    ty.less_than_or_equal = Some(Value::Native(
        |i, args| Value::Bool(args[0].int(i) <= args[1].as_int(i)),
        ArgPattern::Exact(2),
    ));

    ty.greater_than_or_equal = Some(Value::Native(
        |i, args| Value::Bool(args[0].int(i) >= args[1].as_int(i)),
        ArgPattern::Exact(2),
    ));

    Ptr::new(ty)
}

fn float_ty(base: Ptr<Type>) -> Ptr<Type> {
    let mut ty = Type::new(Ptr::new("float".to_string()), base);

    ty.display = Some(Value::Native(
        |i, args| Value::String(Ptr::new(format!("{}", args[0].float(i)))),
        ArgPattern::Exact(1),
    ));

    ty.add = Some(Value::Native(
        |i, args| Value::Float(args[0].float(i) + args[1].as_float(i)),
        ArgPattern::Exact(2),
    ));

    ty.subtract = Some(Value::Native(
        |i, args| Value::Float(args[0].float(i) - args[1].as_float(i)),
        ArgPattern::Exact(2),
    ));

    ty.divide = Some(Value::Native(
        |i, args| Value::Float(args[0].float(i) / args[1].as_float(i)),
        ArgPattern::Exact(2),
    ));

    ty.multiply = Some(Value::Native(
        |i, args| Value::Float(args[0].float(i) * args[1].as_float(i)),
        ArgPattern::Exact(2),
    ));

    ty.less_than = Some(Value::Native(
        |i, args| Value::Bool(args[0].float(i) < args[1].as_float(i)),
        ArgPattern::Exact(2),
    ));

    ty.greater_than = Some(Value::Native(
        |i, args| Value::Bool(args[0].float(i) > args[1].as_float(i)),
        ArgPattern::Exact(2),
    ));

    ty.less_than_or_equal = Some(Value::Native(
        |i, args| Value::Bool(args[0].float(i) <= args[1].as_float(i)),
        ArgPattern::Exact(2),
    ));

    ty.greater_than_or_equal = Some(Value::Native(
        |i, args| Value::Bool(args[0].float(i) >= args[1].as_float(i)),
        ArgPattern::Exact(2),
    ));

    Ptr::new(ty)
}

fn function_ty(base: Ptr<Type>) -> Ptr<Type> {
    let mut ty = Type::new(Ptr::new("function".to_string()), base);

    ty.display = Some(Value::Native(
        |_i, _args| Value::String(Ptr::new("<function object>".to_string())),
        ArgPattern::Exact(1),
    ));

    Ptr::new(ty)
}

fn bool_ty(base: Ptr<Type>) -> Ptr<Type> {
    let mut ty = Type::new(Ptr::new("bool".to_string()), base);

    ty.display = Some(Value::Native(
        |i, args| Value::String(Ptr::new(format!("{}", args[0].bool(i)))),
        ArgPattern::Exact(1),
    ));

    Ptr::new(ty)
}

fn frame_ty(base: Ptr<Type>) -> Ptr<Type> {
    let mut ty = Type::new(Ptr::new("Frame".to_string()), base);

    ty.display = Some(Value::Native(
        |_i, _args| Value::String(Ptr::new("<frame object>".to_string())),
        ArgPattern::Exact(1),
    ));

    Ptr::new(ty)
}

fn native_ty(base: Ptr<Type>) -> Ptr<Type> {
    let mut ty = Type::new(Ptr::new("NativeFunction".to_string()), base);

    ty.display = Some(Value::Native(
        |_i, _args| Value::String(Ptr::new("<native function>".to_string())),
        ArgPattern::Exact(1),
    ));

    Ptr::new(ty)
}

fn code_ty(base: Ptr<Type>) -> Ptr<Type> {
    let mut ty = Type::new(Ptr::new("Code".to_string()), base);

    ty.display = Some(Value::Native(
        |_i, _args| Value::String(Ptr::new("<code object>".to_string())),
        ArgPattern::Exact(1),
    ));

    Ptr::new(ty)
}

fn null_ty(base: Ptr<Type>) -> Ptr<Type> {
    let mut ty = Type::new(Ptr::new("null".to_string()), base);

    ty.display = Some(Value::Native(
        |_i, _args| Value::String(Ptr::new("null".to_string())),
        ArgPattern::Exact(1),
    ));

    Ptr::new(ty)
}

fn object_ty() -> Ptr<Type> {
    let mut ty = Type::root(Ptr::new("object".to_string()));

    ty.display = Some(Value::Native(
        |_i, _args| Value::String(Ptr::new("<object>".to_string())),
        ArgPattern::Exact(1),
    ));

    ty.equals = Some(Value::Native(
        |_i, args| Value::Bool(equals(&args[0], &args[1])),
        ArgPattern::Exact(2),
    ));

    ty.not_equals = Some(Value::Native(
        |_i, args| Value::Bool(!equals(&args[0], &args[1])),
        ArgPattern::Exact(2),
    ));

    ty.get_property = Some(Value::Native(
        |i, args| get_property(i, &args[0], &args[1].string(i)),
        ArgPattern::Exact(2),
    ));

    ty.set_property = Some(Value::Native(
        |i, args| {
            set_property(i, &args[0], args[1].string(i), args[2].clone());
            Value::Null
        },
        ArgPattern::Exact(3),
    ));

    Ptr::new(ty)
}

fn set_property(interp: &Interpreter, target: &Value, prop: Ptr<String>, value: Value) {
    if let Value::Object(obj) = target {
        obj.value_mut().set_property(prop, value);
    } else {
        panic!(
            "object of type `{}` does not support `$set_property`",
            target.ty(interp.builtins()).value().name.value()
        );
    }
}

fn get_property(interp: &Interpreter, target: &Value, prop: &Ptr<String>) -> Value {
    if let Value::Object(obj) = target {
        match obj.value().get_property(prop) {
            Some(res @ Value::Function(_)) | Some(res @ Value::Native(..)) => {
                return Value::Bound(Ptr::new(res), Ptr::new(target.clone()));
            }
            Some(value) => return value,
            _ => {}
        }
    }

    let mut ty = target.ty(interp.builtins());

    loop {
        match ty.value().properties.get(prop) {
            Some(res @ Value::Function(_)) | Some(res @ Value::Native(..)) => {
                return Value::Bound(Ptr::new(res.clone()), Ptr::new(target.clone()));
            }
            Some(value) => return value.clone(),
            _ => {}
        }

        if let Some(base) = ty.value().base.clone() {
            ty = base;
        } else {
            panic!("object has no property `{}`", prop.value());
        }
    }
}

fn equals(lhs: &Value, rhs: &Value) -> bool {
    match (lhs, rhs) {
        (Value::Bool(l), Value::Bool(r)) => l == r,
        (Value::Bool(l), Value::Integer(r)) => *l == (*r != 0),
        (Value::Bool(l), Value::Float(r)) => *l == (*r != 0.0),
        (Value::Integer(l), Value::Integer(r)) => l == r,
        (Value::Integer(l), Value::Float(r)) => *l as f64 == *r,
        (Value::Integer(l), Value::Bool(r)) => (*l != 0) == *r,
        (Value::Float(l), Value::Float(r)) => l == r,
        (Value::Float(l), Value::Integer(r)) => *l == *r as f64,
        (Value::Float(l), Value::Bool(r)) => (*l != 0.0) == *r,
        (Value::Type(l), Value::Type(r)) => l.id() == r.id(),
        (Value::String(l), Value::String(r)) => (l.id() == r.id()) || (l.value() == r.value()),
        (Value::Object(l), Value::Object(r)) => l.id() == r.id(),
        (Value::Null, Value::Null) => true,
        (Value::Native(l, _), Value::Native(r, _)) => *l as usize == *r as usize,
        _ => false,
    }
}

fn ty_ty(base: Ptr<Type>) -> Ptr<Type> {
    let mut ty = Type::new(Ptr::new("object".to_string()), base);

    ty.display = Some(Value::Native(
        |_i, _args| Value::String(Ptr::new("<type object>".to_string())),
        ArgPattern::Exact(1),
    ));

    Ptr::new(ty)
}
