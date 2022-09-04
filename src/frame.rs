use std::collections::HashMap;

use crate::instruction::ConstantIndex;
use crate::instruction::Instruction;
use crate::instruction::NameIndex;
use crate::ptr::{Ptr, PtrMut};
use crate::value::Value;

pub struct Function {
    code: Ptr<Code>,
    outer_frame: PtrMut<Frame>,
}

impl Function {
    pub fn new(code: Ptr<Code>, outer_frame: PtrMut<Frame>) -> Self {
        Self { code, outer_frame }
    }

    pub fn as_frame(&self, calling_frame: PtrMut<Frame>) -> Frame {
        Frame::new(
            self.code.clone(),
            Some(calling_frame),
            Some(self.outer_frame.clone()),
        )
    }
}

pub struct Code {
    instructions: Vec<Instruction>,
    constants: Vec<Value>,
    names: Vec<Ptr<String>>,
    parameters: usize,
}

impl Code {
    pub fn new(
        instructions: Vec<Instruction>,
        constants: Vec<Value>,
        names: Vec<Ptr<String>>,
        parameters: usize,
    ) -> Self {
        Self {
            instructions,
            constants,
            names,
            parameters,
        }
    }
}

pub struct Frame {
    instruction_count: usize,
    code: Ptr<Code>,
    scope: HashMap<Ptr<String>, Value>,
    calling_frame: Option<PtrMut<Frame>>,
    outer_frame: Option<PtrMut<Frame>>,
    stack: Vec<Value>,
}

impl Frame {
    pub fn new(
        code: Ptr<Code>,
        calling_frame: Option<PtrMut<Frame>>,
        outer_frame: Option<PtrMut<Frame>>,
    ) -> Self {
        Self {
            instruction_count: 0,
            calling_frame,
            outer_frame,
            scope: HashMap::new(),
            stack: Vec::new(),
            code,
        }
    }

    pub fn scope(&self) -> &HashMap<Ptr<String>, Value> {
        &self.scope
    }

    pub fn parameters(&self) -> Vec<Ptr<String>> {
        let count = self.code.value().parameters;
        self.code.value().names[0..count].to_vec()
    }

    pub fn calling_frame(&self) -> Option<PtrMut<Frame>> {
        self.calling_frame.clone()
    }

    pub fn outer_frame(&self) -> Option<PtrMut<Frame>> {
        self.outer_frame.clone()
    }

    pub fn constant(&self, consi: ConstantIndex) -> Value {
        self.code.value().constants[consi].clone()
    }

    pub fn name(&self, namei: NameIndex) -> Ptr<String> {
        self.code.value().names[namei].clone()
    }

    pub fn variable(&self, name: &Ptr<String>) -> Option<&Value> {
        self.scope.get(name)
    }

    pub fn variable_mut(&mut self, name: &Ptr<String>) -> Option<&mut Value> {
        self.scope.get_mut(name)
    }

    pub fn declare(&mut self, name: Ptr<String>, value: Value) {
        self.scope.insert(name, value);
    }

    pub fn contains_variable(&self, name: &Ptr<String>) -> bool {
        self.scope.contains_key(name)
    }

    pub fn instruction(&self) -> Instruction {
        self.code.value().instructions[self.instruction_count]
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn jump_relative(&mut self, count: isize) {
        if count >= 0 {
            self.instruction_count += count as usize;
        } else {
            self.instruction_count -= count as usize;
        }
    }

    pub fn jump_absolute(&mut self, addr: usize) {
        self.instruction_count = addr;
    }

    pub fn pop(&mut self) -> Value {
        self.stack
            .pop()
            .expect("stack corruption: no values left to pop")
    }

    pub fn pop_pair(&mut self) -> (Value, Value) {
        (self.pop(), self.pop())
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: Iterator<Item = Value>,
    {
        self.stack.extend(iter)
    }
}
