//! Types for SMT encoding

use anyhow::Error;
use std::{
    collections::HashMap, 
    fmt::Display,
};
use rsmt2::print::{
    Expr2Smt, 
    Sort2Smt,
};

#[derive(Debug, Clone)]
enum Expr {
    Value(Value),
    CValue(&'static str),
    Symb(String),
    CSymb(&'static str),
    App(Vec<Expr>),
}

impl Expr {
    #[allow(unused)]
    fn to_string(&self) -> String {
        let mut buf = Vec::new();
        self.expr_to_smt2(&mut buf, ()).unwrap();
        String::from_utf8(buf).unwrap()
    }
}

impl Expr2Smt for Expr {
    fn expr_to_smt2<Writer>(&self, w: &mut Writer, info: ()) -> rsmt2::SmtRes<()>
    where
        Writer: std::io::Write,
    {
        match self {
            Expr::Value(v) => v.expr_to_smt2(w, info)?,
            Expr::CValue(v) => write!(w, "{v}")?,
            Expr::Symb(s) => write!(w, "{s}")?,
            Expr::CSymb(s) => write!(w, "{s}")?,
            Expr::App(vs) => {
                write!(w, "(")?;
                for (i, v) in vs.iter().enumerate() {
                    if i != 0 {
                        write!(w, " ")?;
                    }
                    v.expr_to_smt2(w, info)?;
                }
                write!(w, ")")?
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FField {
    expr: Expr,
}

impl FField {
    pub(crate) fn new_value(value: &str) -> Self {
        Self { expr: Expr::Value(Value::FField(value.to_string())) }
    }

    pub(crate) fn new_const(name: &str) -> Self {
        Self { expr: Expr::Symb(name.to_string()) }
    }
    
    pub(crate) fn zero() -> Self {
        Self { expr: Expr::CValue("(as ff0 FF)") }
    }

    pub(crate) fn one() -> Self {
        Self { expr: Expr::CValue("(as ff1 FF)") }
    }

    pub(crate) fn add(self, other: Self) -> Self {
        let expr = Expr::App(vec![Expr::CSymb("ff.add"), self.expr, other.expr]);
        Self { expr }
    }

    pub(crate) fn radd(ops: Vec<Self>) -> Self {
        Self::replicated_op(ops, "ff.add")
    }

    pub(crate) fn mul(self, other: Self) -> Self {
        let expr = Expr::App(vec![Expr::CSymb("ff.mul"), self.expr, other.expr]);
        Self { expr }
    }

    pub(crate) fn rmul(ops: Vec<Self>) -> Self {
        Self::replicated_op(ops, "ff.mul")
    }

    pub(crate) fn rbitsum(ops: Vec<Self>) -> Self {
        Self::replicated_op(ops, "ff.bitsum")
    }

    fn replicated_op(ops: Vec<Self>, op: &'static str) -> Self {
        assert!(ops.len() >= 2);
        let mut vec = vec![Expr::CSymb(op)];
        for other in ops {
            vec.push(other.expr);
        }
        let expr = Expr::App(vec);
        Self { expr }
    }

    pub(crate) fn eq(self, other: Self) -> Bool {
        let expr = Expr::App(vec![Expr::CSymb("="), self.expr, other.expr]);
        Bool { expr }
    }
}

#[derive(Debug, Clone)]
pub struct Bool {
    expr: Expr,
}

impl Bool {
    pub fn new_value(value: bool) -> Self {
        Self { expr: Expr::Value(Value::Bool(value)) }
    }

    pub fn new_const(name: &str) -> Self {
        Self { expr: Expr::Symb(name.to_string()) }
    }

    pub fn and(self, other: Self) -> Self {
        let expr = Expr::App(vec![Expr::CSymb("and"), self.expr, other.expr]);
        Self { expr }
    }

    pub fn or(self, other: Self) -> Self {
        let expr = Expr::App(vec![Expr::CSymb("or"), self.expr, other.expr]);
        Self { expr }
    }

    pub fn imp(self, other: Self) -> Self {
        let expr = Expr::App(vec![Expr::CSymb("=>"), self.expr, other.expr]);
        Self { expr }
    }

    pub fn neg(self) -> Self {
        let expr = Expr::App(vec![Expr::CSymb("not"), self.expr]);
        Self { expr }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct Int {
    expr: Expr,
}
impl Int {
    pub(crate) fn new_value(value: &str) -> Self {
        Self { expr: Expr::Value(Value::Int(value.to_string())) }
    }

    pub(crate) fn new_const(name: &str) -> Self {
        Self { expr: Expr::Symb(name.to_string()) }
    }

    pub(crate) fn zero() -> Self {
        Self { expr: Expr::Value(Value::Int("0".to_string())) }
    }

    pub(crate) fn one() -> Self {
        Self { expr: Expr::Value(Value::Int("1".to_string())) }
    }

    #[allow(unused)]
    pub(crate) fn add(self, other: Self) -> Self {
        let expr = Expr::App(vec![Expr::CSymb("+"), self.expr, other.expr]);
        Self { expr }
    }

    #[allow(unused)]
    pub(crate) fn mul(self, other: Self) -> Self {
        let expr = Expr::App(vec![Expr::CSymb("*"), self.expr, other.expr]);
        Self { expr }
    }

    pub(crate) fn radd(ops: Vec<Self>) -> Self {
        Self::replicated_op(ops, "+")
    }

    pub(crate) fn rmul(ops: Vec<Self>) -> Self {
        Self::replicated_op(ops, "*")
    }

    fn replicated_op(ops: Vec<Self>, op: &'static str) -> Self {
        assert!(ops.len() >= 2);
        let mut vec = vec![Expr::CSymb(op)];
        for other in ops {
            vec.push(other.expr);
        }
        let expr = Expr::App(vec);
        Self { expr }
    }

    pub(crate) fn modu(self, other: Self) -> Self {
        let expr = Expr::App(vec![Expr::CSymb("mod"), self.expr, other.expr]);
        Self { expr }
    }

    pub(crate) fn lt(self, other: Self) -> Bool {
        let expr = Expr::App(vec![Expr::CSymb("<"), self.expr, other.expr]);
        Bool { expr }
    }

    #[allow(unused)]
    pub(crate) fn gte(self, other: Self) -> Bool {
        let expr = Expr::App(vec![Expr::CSymb(">="), self.expr, other.expr]);
        Bool { expr }
    }

    pub(crate) fn eq(self, other: Self) -> Bool {
        let expr = Expr::App(vec![Expr::CSymb("="), self.expr, other.expr]);
        Bool { expr }
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Bool,
    Int,
    FField,
}

impl Sort2Smt for Type {
    fn sort_to_smt2<Writer>(&self, w: &mut Writer) -> rsmt2::SmtRes<()>
    where
        Writer: std::io::Write,
    {
        match self {
            Type::Bool => write!(w, "Bool")?,
            Type::Int => write!(w, "Int")?,
            Type::FField => write!(w, "FF")?,
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(String),
    FField(String),
}

impl Expr2Smt for Value {
    fn expr_to_smt2<Writer>(&self, w: &mut Writer, _i: ()) -> rsmt2::SmtRes<()>
    where
        Writer: std::io::Write,
    {
        match self {
            Value::Bool(b) => write!(w, "{}", if *b { "true" } else { "false" })?,
            Value::Int(i) => write!(w, "{}", i)?,
            Value::FField(f) => write!(w, "(as ff{} FF)", f)?,
        };
        Ok(())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Int(i) => write!(f, "{}", i),
            Value::FField(ff) => write!(f, "{}", ff),
        }
    }
}

#[derive(Debug)]
pub enum SolverOutput {
    Sat,
    Unsat,
    Unknown,
}

pub struct Solver {
    rsmt: rsmt2::Solver<()>,
    prime: &'static str,
}

impl Solver {
    fn new_with_options(prime: &'static str, options: Vec<&'static str>) -> Self {
        let mut conf = rsmt2::SmtConf::cvc5("cvc5");
        conf.models();
        conf.incremental();
        conf.unsat_cores();
        for option in options {
            conf.option(option);
        }
        let rsmt = rsmt2::Solver::new(conf, ()).unwrap();

        // Uncomment the following two lines, and make `rsmt` mutable to 
        // write the SMT queries to a file for debugging
        // let file = std::fs::File::create("out.smt2").unwrap();
        // rsmt.tee(file).unwrap();

        Self { rsmt, prime }
    }

    fn new(prime: &'static str) -> Self {
        Self::new_with_options(prime, vec![])
    }

    pub fn new_int(prime: &'static str) -> Self {
        let mut solver = Self::new(prime);
        solver.rsmt.set_custom_logic("QF_NIA").unwrap();
        solver
    }

    pub fn new_ff(prime: &'static str, is_split: bool) -> Self {
        let mut solver = Self::new(prime);
        solver.rsmt.set_custom_logic("QF_FF").unwrap();
        if is_split {
            solver.rsmt.set_option(":ff-solver", "split").unwrap();
        } else {
            solver.rsmt.set_option(":ff-solver", "gb").unwrap();
        }
        solver.rsmt.define_sort("FF", &[""], format!("(_ FiniteField {})", prime)).unwrap();
        solver
    }

    pub fn set_option(&mut self, option: &str, value: &str) {
        self.rsmt.set_option(option, value).unwrap();
    }

    pub fn new_ff_gb(prime: &'static str) -> Self {
        Self::new_ff(prime, false)
    }

    pub fn new_ff_split(prime: &'static str) -> Self {
        Self::new_ff(prime, true)
    }

    pub fn assert(&mut self, b_expr: Bool) {
        self.rsmt.assert(b_expr.expr).unwrap();
    }

    pub fn check_sat_assuming(&mut self, actlits: &String) -> Result<SolverOutput, Error> {
        let result = match self.rsmt.check_sat_assuming_or_unk::<&String, Option<&String>>(Some(actlits)).map_err(|e| Error::msg(e.to_string()))? {
            Some(true) => Ok(SolverOutput::Sat),
            Some(false) => Ok(SolverOutput::Unsat),
            None => Ok(SolverOutput::Unknown),
        };
        result
    }

    pub fn push(&mut self) {
        self.rsmt.push(1).unwrap();
    }
    
    pub fn pop(&mut self) {
        self.rsmt.pop(1).unwrap();
    }

    pub fn check_sat(&mut self) -> Result<SolverOutput, Error> {
        let result = match self.rsmt.check_sat_or_unk().map_err(|e| Error::msg(e.to_string()))? {
            Some(true) => Ok(SolverOutput::Sat),
            Some(false) => Ok(SolverOutput::Unsat),
            None => Ok(SolverOutput::Unknown),
        };
        result
    }
    
    pub fn get_model(&mut self) -> HashMap<String, Value> {
        let model = self.rsmt.get_model().unwrap();
        let mut res = HashMap::new();
        for (id, _, typ, val) in model {
            let val = if typ == "Int" {
                Value::Int(val.to_string())
            } else if typ == "Bool" {
                let b = match val.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => panic!("Unexpected Bool value in model: {}", val),
                };
                Value::Bool(b)
            } else if typ.starts_with("(_ FiniteField") {
                // val is of the form (as ff123 FF)
                let val = val.split_once("m").unwrap().0.strip_prefix("#f").unwrap();
                Value::FField(val.to_string())
            } else {
                panic!("Unexpected type in model: {}", typ);
            };
            res.insert(id, val);
        }
        res
    }
    
    pub fn declare_const(&mut self, symbol: &str, typ: Type) {
        self.rsmt.declare_const(symbol, typ).map_err(|e| Error::msg(e.to_string())).unwrap();
    }

    pub fn prime(&self) -> &'static str {
        self.prime
    }
}

mod tests {
    #[test]
    fn test_solver() {
        let mut solver = super::Solver::new_ff_gb("13");
        solver.declare_const("x", super::Type::FField);
        solver.declare_const("y", super::Type::FField);
        let x = super::FField::new_const("x");
        let y = super::FField::new_const("y");
        let ff_one = super::FField::one();
        let constraint = x.add(y).eq(ff_one);
        solver.assert(constraint);
        let checksat = solver.check_sat();
        match checksat.unwrap() {
            super::SolverOutput::Sat => {
                let model = solver.get_model();
                println!("SAT with model: {model:?}");
            }
            super::SolverOutput::Unsat => {
                println!("UNSAT");
            }
            super::SolverOutput::Unknown => {
                println!("UNKNOWN");
            }
        }
    }
}
