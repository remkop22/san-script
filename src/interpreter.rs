use std::process::exit;

use crate::{
    builtins::Builtins,
    frame::{Frame, Function},
    instruction::{ConstantIndex, Instruction, NameIndex},
    ptr::{Ptr, PtrMut},
    value::{ArgPattern, Value},
};

enum Action {
    Call(PtrMut<Frame>),
    Return(Value),
    ReturnNative(Value),
}

pub struct Interpreter {
    frame: PtrMut<Frame>,
    builtins: Builtins,
    next_action: Option<Action>,
}

#[macro_export]
macro_rules! get_native_prop {
    ($interp:ident, $obj:expr, $prop:ident) => {{
        let mut ty = $obj.ty($interp.builtins());

        loop {
            if let Some(prop) = ty.value().$prop.clone() {
                break prop;
            }

            if let Some(base) = ty.value().base.clone() {
                ty = base;
            } else {
                panic!(
                    "value of type `{}` does not support `${}`",
                    $obj.ty($interp.builtins()).value().name.value(),
                    stringify!($prop)
                )
            }
        }
    }};
}

macro_rules! operation {
    ($prop:ident) => {
        fn $prop(&mut self) {
            let (lhs, rhs) = self.frame.value_mut().pop_pair();
            let prop = get_native_prop!(self, rhs, $prop);
            self.call_value(prop, &[lhs, rhs]);
        }
    };
}

impl Interpreter {
    /// Create a new interpreter with a root frame and builtins instance.
    pub fn new(frame: PtrMut<Frame>, builtins: Builtins) -> Self {
        Self {
            frame,
            builtins,
            next_action: None,
        }
    }

    /// A reference to the interpreter Builtins.
    pub fn builtins(&self) -> &Builtins {
        &self.builtins
    }

    /// Run interpreter till it halts.
    pub fn run(&mut self) {
        loop {
            let return_value = self.run_frame();
            self.frame.value_mut().push(return_value);
        }
    }

    /// Call this value and run the interpreter till it returns.
    pub fn call_with_return(&mut self, value: Value, args: &[Value]) -> Value {
        self.call_value(value, args);
        self.run_frame()
    }

    fn run_frame(&mut self) -> Value {
        loop {
            if let Some(action) = self.next_action.take() {
                match action {
                    Action::Call(frame) => self.frame = frame,
                    Action::ReturnNative(value) => return value,
                    Action::Return(value) => {
                        let caller = self.frame.value().calling_frame();
                        if let Some(caller) = caller {
                            self.frame = caller;
                            return value;
                        } else {
                            self.exit(0);
                        }
                    }
                }
            }

            self.execute();
        }
    }

    fn pop(&self) {
        self.frame.value_mut().pop();
    }

    fn call(&mut self, argc: usize) {
        let func = self.frame.value_mut().pop();
        let args: Vec<_> = (0..argc).map(|_| self.frame.value_mut().pop()).collect();
        self.call_value(func, &args);
    }

    fn call_value(&mut self, value: Value, args: &[Value]) {
        match value {
            Value::Function(func) => {
                let mut frame = func.value().as_frame(self.frame.clone());
                let parameters = frame.parameters();

                if parameters.len() != args.len() {
                    panic!(
                        "expected {} args, but found {}",
                        parameters.len(),
                        args.len()
                    );
                }

                for (param, value) in parameters.into_iter().zip(args) {
                    frame.declare(param, value.clone());
                }

                self.next_action = Some(Action::Call(PtrMut::new(frame)));
            }
            Value::Native(func, params) => {
                let ret_val = match params {
                    ArgPattern::Exact(len) if len != args.len() => {
                        panic!("expected {} args, but found {}", len, args.len())
                    }
                    ArgPattern::Min(min) if args.len() < min => {
                        panic!("expected at least {} args, but found {}", min, args.len())
                    }
                    ArgPattern::Max(max) if args.len() > max => {
                        panic!(
                            "expected not more than {} args, but found {}",
                            max,
                            args.len()
                        )
                    }
                    ArgPattern::Range(min, max) if (args.len() < min || args.len() > max) => {
                        panic!(
                            "expect between {} and {} args, but found {}",
                            min,
                            max,
                            args.len()
                        )
                    }
                    _ => func(self, args),
                };

                self.next_action = Some(Action::ReturnNative(ret_val));
            }
            Value::Bound(obj, bound) => {
                let mut new_args = vec![bound.value().clone()];
                new_args.extend_from_slice(args);
                self.call_value(obj.value().clone(), &new_args)
            }
            _ => {
                let ty = value.ty(&self.builtins);

                let call = ty.value().call.clone().unwrap_or_else(|| {
                    panic!(
                        "object of type `{}` does not support `$call`",
                        ty.value().name.value()
                    )
                });

                let mut new_args = vec![value];
                new_args.extend_from_slice(args);
                self.call_value(call, &new_args);
            }
        }
    }

    fn create_function(&self) {
        let value = self.frame.value_mut().pop();

        if let Value::Code(code) = value {
            let func = Ptr::new(Function::new(code, self.frame.clone()));
            self.frame.value_mut().push(Value::Function(func));
        } else {
            panic!("invalid value, expected code object");
        }
    }

    fn load_constant(&self, consi: ConstantIndex) {
        let constant = self.frame.value().constant(consi);
        self.frame.value_mut().push(constant);
    }

    fn resolve_name(&self, name: &Ptr<String>) -> Option<PtrMut<Frame>> {
        let mut cur_frame = Some(self.frame.clone());

        while let Some(frame) = cur_frame {
            if frame.value().contains_variable(name) {
                return Some(frame);
            }

            cur_frame = frame.value().outer_frame();
        }

        None
    }

    fn load_variable(&self, namei: NameIndex) {
        let name = self.frame.value().name(namei);

        if let Some(builtin) = self.builtins.resolve(&*name.value()) {
            self.frame.value_mut().push(builtin);
        } else if let Some(frame) = self.resolve_name(&name) {
            let value = frame.value().variable(&name).unwrap().clone();
            self.frame.value_mut().push(value);
        } else {
            panic!("couln't resolve variable named `{}`", name.value());
        }
    }

    fn store_variable(&self, namei: NameIndex) {
        let name = self.frame.value().name(namei);
        if let Some(frame) = self.resolve_name(&name) {
            let value = self.frame.value_mut().pop();
            *frame.value_mut().variable_mut(&name).unwrap() = value;
        } else {
            panic!("couln't resolve variable named `{}`", name.value());
        }
    }

    fn declare(&self, namei: NameIndex) {
        let name = self.frame.value().name(namei);
        let value = self.frame.value_mut().pop();
        self.frame.value_mut().declare(name, value);
    }

    fn create_list(&self, len: usize) {
        let mut list = Vec::new();

        for _ in 0..len {
            list.push(self.frame.value_mut().pop());
        }

        self.frame.value_mut().push(Value::List(PtrMut::new(list)));
    }

    fn execute(&mut self) {
        let instruction = self.frame.value().instruction();

        match instruction {
            Instruction::Pop => self.pop(),
            Instruction::Call(argc) => self.call(argc),
            Instruction::Return => self.return_statement(),
            Instruction::CreateFunction => self.create_function(),
            Instruction::LoadConstant(consi) => self.load_constant(consi),
            Instruction::LoadVariable(namei) => self.load_variable(namei),
            Instruction::StoreVariable(namei) => self.store_variable(namei),
            Instruction::Declare(namei) => self.declare(namei),
            Instruction::Exit(code) => self.exit(code),
            Instruction::StoreSubscript => self.store_subscript(),
            Instruction::LoadSubscript => self.load_subscript(),
            Instruction::StoreProperty(namei) => self.store_property(namei),
            Instruction::LoadProperty(namei) => self.load_property(namei),
            Instruction::Add => self.add(),
            Instruction::Subtract => self.subtract(),
            Instruction::Divide => self.divide(),
            Instruction::Multiply => self.multiply(),
            Instruction::Equals => self.equals(),
            Instruction::NotEquals => self.not_equals(),
            Instruction::GreaterThan => self.greater_than(),
            Instruction::GreaterThanOrEqual => self.greater_than_or_equal(),
            Instruction::LessThan => self.less_than(),
            Instruction::LessThanOrEqual => self.less_than_or_equal(),
            Instruction::Jump(jmp) => self.jump(jmp),
            Instruction::JumpFalse(jmp) => self.jump_false(jmp),
            Instruction::CreateList(len) => self.create_list(len),
        }

        self.frame.value_mut().jump_relative(1);
    }

    fn jump(&self, jmp: usize) {
        self.frame.value_mut().jump_absolute(jmp);
    }

    fn jump_false(&self, jmp: usize) {
        if !self.frame.value_mut().pop().as_bool(self) {
            self.frame.value_mut().jump_absolute(jmp);
        }
    }

    operation!(add);
    operation!(subtract);
    operation!(multiply);
    operation!(divide);
    operation!(equals);
    operation!(not_equals);
    operation!(less_than);
    operation!(greater_than);
    operation!(greater_than_or_equal);
    operation!(less_than_or_equal);

    pub fn store_subscript(&mut self) {
        let obj = self.frame.value_mut().pop();
        let subs = self.frame.value_mut().pop();
        let value = self.frame.value_mut().pop();

        let set_subscript = get_native_prop!(self, obj, set_subscript);
        self.call_value(set_subscript, &[obj, subs, value]);
    }

    pub fn load_subscript(&mut self) {
        let obj = self.frame.value_mut().pop();
        let subs = self.frame.value_mut().pop();

        let get_subscript = get_native_prop!(self, obj, get_subscript);
        self.call_value(get_subscript, &[obj, subs]);
    }

    pub fn store_property(&mut self, namei: NameIndex) {
        let obj = self.frame.value_mut().pop();
        let prop = self.frame.value().name(namei);
        let value = self.frame.value_mut().pop();

        let set_property = get_native_prop!(self, obj, set_property);
        self.call_value(set_property, &[obj, Value::String(prop), value]);
    }

    pub fn load_property(&mut self, namei: NameIndex) {
        let obj = self.frame.value_mut().pop();
        let prop = self.frame.value().name(namei);

        let get_property = get_native_prop!(self, obj, get_property);
        self.call_value(get_property, &[obj, Value::String(prop)]);
    }

    fn exit(&mut self, code: usize) {
        exit(code as i32)
    }

    fn return_statement(&mut self) {
        let ret_val = self.frame.value_mut().pop();
        self.next_action = Some(Action::Return(ret_val));
    }
}
