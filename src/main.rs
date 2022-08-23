#![allow(unused, dead_code)]
use mlua::prelude::*;
use std::{fs::read_to_string, collections::BTreeMap};
use fasteval;
use fasteval::Compiler;  // import this trait so we can call compile().
use fasteval::Evaler;    // import this trait so we can call eval().

struct LuaVectorF64Parser {
    compiled: fasteval::Instruction,
    slab: fasteval::Slab,
    map: BTreeMap<String, f64>,
    vec: LuaVectorF64,
}

impl LuaVectorF64Parser {
    fn new(expr_str: &str, vec: LuaVectorF64) -> Result<Self, fasteval::Error> {
        let parser = fasteval::Parser::new();
        let mut slab = fasteval::Slab::new();
        let mut map = BTreeMap::new();
    
        let compiled = parser.parse(expr_str, &mut slab.ps)?.from(&slab.ps).compile(&slab.ps, &mut slab.cs);
        Ok(LuaVectorF64Parser{ compiled, slab, map, vec})
    }

    fn eval(&mut self) -> Result<(), fasteval::Error> {
        for item in self.vec.data.iter_mut() {
            self.map.insert("x".to_string(), *item);
            let val = self.compiled.eval(&self.slab, &mut self.map)?;
            *item = val;
        }

        Ok(())
    }
}

impl LuaUserData for LuaVectorF64Parser {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("eval", |lua_ctx, object, ()| {
            object.eval().unwrap();
            Ok(object.vec.clone())
        });

        methods.add_method("get", |lua_ctx, object, ()| {
            Ok(LuaVectorF64::from_vec(object.vec.data.clone()))
        });
    }
}

#[derive(Clone)]
struct LuaVectorF64 {
    data: Vec<f64>
}

impl LuaVectorF64 {
    fn new() -> Self {
        Self {data: Vec::new()}
    }

    fn from_vec(data: Vec<f64>) -> Self {
        Self {data}
    }

    fn range(&mut self, min: f64, max: f64, mut step: f64) -> () {
        let mut i = min;
        step = step.abs();

        if max > min {
            self.reserve((max-min) as usize);
            while i < max {
                self.data.push(i);
                i += step;
            }
        } else if min > max {
            self.reserve((min-max) as usize);
            i = min;
            while i > max {
                self.data.push(i);
                i -= step;
            }
        }
    }



    fn slice(&self, mut min: usize, mut max: usize, step: usize) -> LuaVectorF64 {
        let mut vec = LuaVectorF64::new();
        let mut i = min;

        let mut i = min;

        if max > min {
            vec.reserve(max-min);
            while i <= max {
                vec.data.push(*self.data.get(i).unwrap());
                i += step;
            }
        } else if min > max {
            vec.reserve(min-max);
            i = min;
            while i >= max {
                dbg!(i);
                vec.data.push(*self.data.get(i).unwrap());
                i -= step;
            }
        }

        vec
    }

    fn to_string(&self) -> String {
        format!("{:?}", self.data)
    }

    fn push(&mut self, value: f64) -> () {
        self.data.push(value)
    }

    fn pop(&mut self) -> f64 {
        self.data.pop().unwrap()
    }

    fn concat(&self, other: &LuaVectorF64) -> LuaVectorF64 {
        let mut vec = LuaVectorF64::new();
        for item in self.data.iter().chain(other.data.iter()) {
            vec.push(*item)
        }
        vec
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn capacity(&self) -> usize {
        self.data.capacity()
    }

    fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    fn index(&self, index: usize) -> f64 {
        *self.data.get(index).unwrap()
    }
}

impl LuaUserData for LuaVectorF64 {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("push", |lua_ctx, object, (value)| {
            Ok(object.push(value))
        });

        methods.add_method("transform", |lua_ctx, object, (expr):(String)| {
            Ok(LuaVectorF64Parser::new(&expr, object.clone()).unwrap())
        });

        methods.add_method_mut("reserve", |lua_ctx, object, (additional)| {
            Ok(object.reserve(additional))
        });

        methods.add_method_mut("pop", |lua_ctx, object, ()| {
            Ok(object.pop())
        });

        methods.add_method("capacity", |lua_ctx, object, ()| {
            Ok(object.capacity())
        });

        methods.add_method_mut("range", |lua_ctx, object, (min, max, step)| {
            Ok(object.range(min, max, step))
        });

        methods.add_meta_function(mlua::MetaMethod::ToString, |lua_ctx, (object):(LuaVectorF64)| {
            Ok(object.to_string())
        });

        methods.add_meta_function(mlua::MetaMethod::Call, |lua_ctx, (object, min, max, step):(LuaVectorF64, usize, usize, usize)| {
            Ok(object.slice(min, max, step))
        });

        methods.add_meta_function(mlua::MetaMethod::Concat, |lua_ctx, (object1, object2):(LuaVectorF64, LuaVectorF64)| {
            Ok(object1.concat(&object2))
        });

        methods.add_meta_function(mlua::MetaMethod::Len, |lua_ctx, (object):(LuaVectorF64)| {
            Ok(object.len())
        });

        methods.add_meta_function(mlua::MetaMethod::Index, |lua_ctx, (object, index):(LuaVectorF64, usize)| {
            Ok(object.index(index))
        });
    }
}

fn main() {
    let lua = mlua::Lua::new();
    let globals = lua.globals();

    let source_code = read_to_string("main.lua").unwrap();

    let cdtk_table = lua.create_table().unwrap();

    let cdtk_vecf64 = lua.create_function(|lua_ctx, ()| {
        Ok(LuaVectorF64::new())
    }).unwrap();
    cdtk_table.set("VecF64", cdtk_vecf64).unwrap();

    globals.set("CDTK", cdtk_table).unwrap();

    lua.load(&source_code).exec().unwrap();
}