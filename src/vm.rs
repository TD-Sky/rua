use std::collections::HashMap;

use crate::{ByteCode, ParseProto, Value};

#[derive(Debug)]
pub struct ExeState {
    globals: HashMap<String, Value>,
    stack: Vec<Value>,
    func_index: usize,
}

impl ExeState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let globals =
            HashMap::from_iter([(String::from("print"), Value::Function(Self::lib_print))]);
        Self {
            globals,
            stack: Vec::new(),
            func_index: 0,
        }
    }

    pub fn execute(&mut self, proto: ParseProto) -> anyhow::Result<()> {
        for code in proto.bytecodes {
            tracing::trace!("executing {code:?}");
            match code {
                ByteCode::GetGlobal(dst, name) => {
                    let key = proto.constants[name as usize].as_str().unwrap();
                    self.set_stack(dst, self.globals.get(key).cloned().unwrap_or_default());
                }
                ByteCode::Move(dst, src) => {
                    self.set_stack(dst, self.stack[src as usize].clone());
                }
                ByteCode::LoadNil(dst) => {
                    self.set_stack(dst, Value::Nil);
                }
                ByteCode::LoadBool(dst, b) => {
                    self.set_stack(dst, Value::Boolean(b));
                }
                ByteCode::LoadInt(dst, i) => {
                    self.set_stack(dst, Value::Integer(i as i64));
                }
                ByteCode::LoadConst(dst, c) => {
                    self.set_stack(dst, proto.constants[c as usize].clone());
                }
                ByteCode::Call(func, _) => {
                    self.func_index = func as usize;
                    let func = match &self.stack[self.func_index] {
                        Value::Function(func) => func,
                        v => anyhow::bail!("{v:?} is not a function"),
                    };
                    func(self);
                }
                ByteCode::SetGlobalConst(gi, ki) => {
                    self.globals.insert(
                        proto.constants[gi as usize].as_str().unwrap().to_owned(),
                        proto.constants[ki as usize].clone(),
                    );
                }
                ByteCode::SetGlobalLocal(gi, src) => {
                    self.globals.insert(
                        proto.constants[gi as usize].as_str().unwrap().to_owned(),
                        self.stack[src as usize].clone(),
                    );
                }
                ByteCode::SetGlobalGlobal(lhsi, rhsi) => {
                    let rhs = self
                        .globals
                        .get(proto.constants[rhsi as usize].as_str().unwrap())
                        .cloned()
                        .unwrap_or_default();
                    self.globals.insert(
                        proto.constants[lhsi as usize].as_str().unwrap().to_owned(),
                        rhs,
                    );
                }
            };
            tracing::trace!("stack: {:#?}", self.stack);
        }

        Ok(())
    }
}

impl ExeState {
    fn set_stack(&mut self, dst: u8, value: Value) {
        if let Some(v) = self.stack.get_mut(dst as usize) {
            *v = value;
        } else {
            self.stack.push(value);
            assert_eq!(dst as usize, self.stack.len() - 1);
        }
    }

    fn lib_print(&mut self) -> i32 {
        println!("{:?}", self.stack[self.func_index + 1]);
        0
    }
}
