use crate::{
    ast::{AssignmentTarget, Expression, Module, Operator, Statement},
    frame::Code,
    instruction::{ConstantIndex, Instruction, NameIndex},
    ptr::Ptr,
    value::Value,
};

#[derive(Debug, PartialEq)]
pub enum Constant {
    Null,
    Integer(i64),
    Float(f64),
    String(String),
    Code(CodeBuilder),
}

impl From<Constant> for Value {
    fn from(c: Constant) -> Self {
        match c {
            Constant::Null => Value::Null,
            Constant::Code(code) => Value::Code(Ptr::new(code.build())),
            Constant::Float(flt) => Value::Float(flt),
            Constant::String(str) => Value::String(Ptr::new(str)),
            Constant::Integer(int) => Value::Integer(int),
        }
    }
}

impl From<i64> for Constant {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}

impl From<f64> for Constant {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<String> for Constant {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<CodeBuilder> for Constant {
    fn from(v: CodeBuilder) -> Self {
        Self::Code(v)
    }
}

#[derive(Debug)]
pub struct CodeBuilder {
    instructions: Vec<Instruction>,
    constants: Vec<Constant>,
    names: Vec<String>,
    parameters: usize,
}

impl CodeBuilder {
    pub fn new(parameters: usize) -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            names: Vec::new(),
            parameters,
        }
    }

    fn use_name(&mut self, name: &str) -> NameIndex {
        if let Some(namei) = self.names.iter().position(|n| n == name) {
            namei
        } else {
            self.names.push(name.to_string());
            self.names.len() - 1
        }
    }

    fn use_constant(&mut self, cons: impl Into<Constant>) -> ConstantIndex {
        let cons = cons.into();

        if let Some(consi) = self.constants.iter().position(|c| c == &cons) {
            consi
        } else {
            self.constants.push(cons);
            self.constants.len() - 1
        }
    }

    pub fn compile_module(&mut self, module: &Module) {
        for stmt in module.body.iter() {
            self.compile_statement(stmt);
        }

        self.inst(Instruction::Exit(0));
    }

    fn inst(&mut self, inst: Instruction) -> usize {
        self.instructions.push(inst);
        self.instructions.len() - 1
    }

    fn compile_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Return(expr) => {
                self.compile_expression(expr);
                self.inst(Instruction::Return);
            }
            Statement::Expression(expr) => {
                self.compile_expression(expr);
                self.inst(Instruction::Pop);
            }
            Statement::Declaration { ident, assign } => {
                if let Some(assign) = assign {
                    self.compile_expression(assign);
                } else {
                    self.compile_constant(Constant::Null);
                }

                let namei = self.use_name(ident);
                self.inst(Instruction::Declare(namei));
            }
            Statement::Assignment { target, source } => self.compile_assignment(target, source),
            Statement::If {
                cond,
                body,
                else_body,
            } => self.compile_if_statement(cond, body, else_body),
        }
    }

    fn compile_operation(&mut self, lhs: &Expression, op: &Operator, rhs: &Expression) {
        self.compile_expression(rhs);
        self.compile_expression(lhs);

        let inst = match op {
            Operator::Add => Instruction::Add,
            Operator::Subtract => Instruction::Subtract,
            Operator::Multiply => Instruction::Multiply,
            Operator::Divide => Instruction::Divide,
            Operator::LessThan => Instruction::LessThan,
            Operator::GreaterThan => Instruction::GreaterThan,
            Operator::LessThanOrEqual => Instruction::LessThanOrEqual,
            Operator::GreaterThanOrEqual => Instruction::GreaterThanOrEqual,
            Operator::Equals => Instruction::Equals,
            Operator::NotEquals => Instruction::NotEquals,
        };

        self.inst(inst);
    }

    fn compile_assignment(&mut self, target: &AssignmentTarget, source: &Expression) {
        self.compile_expression(source);
        match target {
            AssignmentTarget::Identifier(ident) => {
                let namei = self.use_name(ident);
                self.inst(Instruction::StoreVariable(namei));
            }
            AssignmentTarget::Property(expr, property) => {
                self.compile_expression(expr);

                let namei = self.use_name(property);
                self.inst(Instruction::StoreProperty(namei));
            }
            AssignmentTarget::Subscript(expr, subscript) => {
                self.compile_expression(subscript);
                self.compile_expression(expr);
                self.inst(Instruction::StoreSubscript);
            }
        }
    }

    fn compile_function(&mut self, params: &[String], body: &[Statement]) {
        let mut code = CodeBuilder::new(params.len());

        for param in params {
            code.use_name(param);
        }

        for stmt in body {
            code.compile_statement(stmt);
        }

        code.compile_constant(Constant::Null);
        code.inst(Instruction::Return);

        self.compile_constant(code);
        self.inst(Instruction::CreateFunction);
    }

    fn compile_call(&mut self, target: &Expression, arguments: &[Expression]) {
        for arg in arguments.iter().rev() {
            self.compile_expression(arg);
        }

        self.compile_expression(target);
        self.inst(Instruction::Call(arguments.len()));
    }

    fn compile_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Operation { lhs, op, rhs } => self.compile_operation(lhs, op, rhs),
            Expression::Integer(int) => self.compile_constant(*int),
            Expression::Float(flt) => self.compile_constant(*flt),
            Expression::String(str) => self.compile_constant(str.clone()),
            Expression::Function { params, body } => self.compile_function(params, body),
            Expression::FunctionCall { target, args } => self.compile_call(target, args),
            Expression::Subscript(expr, subscript) => {
                self.compile_expression(subscript);
                self.compile_expression(expr);
                self.inst(Instruction::LoadSubscript);
            }
            Expression::Property(expr, property) => {
                self.compile_expression(expr);

                let namei = self.use_name(property);
                self.inst(Instruction::LoadProperty(namei));
            }
            Expression::Object(_items) => todo!(),
            Expression::List(items) => self.compile_list(items),
            Expression::Identifier(ident) => {
                let namei = self.use_name(ident);
                self.inst(Instruction::LoadVariable(namei));
            }
        }
    }

    fn compile_list(&mut self, items: &[Expression]) {
        for item in items.iter().rev() {
            self.compile_expression(item);
        }

        self.inst(Instruction::CreateList(items.len()));
    }

    fn compile_if_statement(
        &mut self,
        cond: &Expression,
        body: &[Statement],
        else_body: &[Statement],
    ) {
        self.compile_expression(cond);

        // If false skip over block
        let label_start = self.inst(Instruction::JumpFalse(0));

        for stmt in body {
            self.compile_statement(stmt);
        }

        let mut label_end = self.instructions.len();

        if !else_body.is_empty() {
            let label_else = self.inst(Instruction::Jump(0));
            label_end += 1;

            for stmt in else_body {
                self.compile_statement(stmt);
            }

            let label_else_end = self.instructions.len();
            *self.instructions.get_mut(label_else).unwrap() = Instruction::Jump(label_else_end - 1);
        }

        *self.instructions.get_mut(label_start).unwrap() = Instruction::JumpFalse(label_end - 1);
    }

    fn compile_constant(&mut self, cons: impl Into<Constant>) {
        let consi = self.use_constant(cons);
        self.inst(Instruction::LoadConstant(consi));
    }

    pub fn build(self) -> Code {
        let constants = self.constants.into_iter().map(|c| c.into()).collect();
        let names = self.names.into_iter().map(Ptr::new).collect();

        Code::new(self.instructions, constants, names, self.parameters)
    }
}

impl PartialEq for CodeBuilder {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
