// Function for running the verifier for ACIR

use crate::{
    encoder::{Translator, num_vars},
    smt::{Bool, SolverOutput, Value},
};
use acir::{FieldElement, circuit::Circuit, native_types::WitnessMap};
use std::collections::HashMap;

pub use crate::error::Error;
pub use crate::smt::Solver;

mod encoder;
mod error;
mod smt;

const _: () = {
    // Ensure that exactly one of the features is selected.
    #[cfg(all(feature = "bn254", not(feature = "bls12_381")))]
    compile_error!("feature \"bn254\" requires feature \"bls12_381\" to be disabled");
};

pub const PRIME: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";

type Model = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub enum Output {
    Falsified(Model),
    Verified,
    Unknown,
}

impl Output {
    pub fn is_verified(&self) -> bool {
        matches!(self, Output::Verified)
    }
    pub fn is_falsified(&self) -> bool {
        matches!(self, Output::Falsified(_))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BackendType {
    FfGb,
    FfSplit,
    Int,
}

impl Default for BackendType {
    fn default() -> Self {
        BackendType::FfGb
    }
}

#[derive(Debug)]
pub struct VerifyResult {
    // The second item of the tuple identifies brillig call opcode location of
    // verification condition
    solver_output: Vec<(Output, usize)>,

    // The first item of the tuple identifies witness ID of the variable to print, 
    // and the second item identifies brillig call opcode location of the verify
    // print statement.
    print_locs: Vec<(u32, usize)>,
}

impl VerifyResult {
    pub fn solver_output(&self) -> &[(Output, usize)] {
        &self.solver_output
    }

    pub fn print_locs(&self) -> &[(u32, usize)] {
        &self.print_locs
    }
}

fn create_solver(backend: BackendType, debug_smt_file: bool) -> Solver {
    match backend {
        BackendType::FfGb => Solver::new_ff_gb(PRIME, debug_smt_file),
        BackendType::FfSplit => Solver::new_ff_split(PRIME,debug_smt_file),
        BackendType::Int => Solver::new_int(PRIME,debug_smt_file),
    }
}

fn use_int(backend: BackendType) -> bool {
    match backend {
        BackendType::FfGb | BackendType::FfSplit => false,
        BackendType::Int => true,
    }
}

fn get_output(solver: &mut Solver, solver_output: SolverOutput) -> Result<Output, Error> {
    match solver_output {
        SolverOutput::Sat => Ok(Output::Falsified(solver.get_model()?)),
        SolverOutput::Unsat => Ok(Output::Verified),
        SolverOutput::Unknown => Ok(Output::Unknown),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RangeOpts{
    limit: Option<u32>,
    kind: RangeEncodingKind,
}

impl RangeOpts {
    pub fn new(limit: Option<u32>, kind: RangeEncodingKind) -> Self {
        Self { limit, kind }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum RangeEncodingKind {
    #[default]
    BitDecomposition,
    BaseDecomposition(u32),
    ExplicitValue,
}

pub fn check_program(
    circuit: &Circuit<FieldElement>,
    brillig_names: Vec<String>,
    backend: BackendType,
    strict: bool,
    range_opts: Option<RangeOpts>,
    debug_smt_file: bool,
) -> Result<VerifyResult, Error> {
    let mut solver = create_solver(backend, debug_smt_file);
    let mut brillig_funcs: HashMap<u32, String> = HashMap::new();
    for (fn_index, name) in brillig_names.clone().into_iter().enumerate() {
        brillig_funcs.insert(fn_index as u32, name);
    }
    let (ver_conds, print_locs) = {
        let use_int = use_int(backend);
        let next_witness_index = num_vars(circuit);
        let mut translator =
            Translator::new(&mut solver, brillig_funcs, next_witness_index, use_int, strict, range_opts);
        translator.translate_to_smt(circuit)?;
        if !translator.has_constraints() {
            println!("Circuit has no constraints");
            return Ok(VerifyResult { solver_output: vec![], print_locs: vec![] });
        }
        (translator.ver_conds(), translator.print_locs())
    };
    let mut outputs = Vec::new();
    for (actlit_name, call_loc) in ver_conds {
        let out_sat = solver.check_sat_assuming(&actlit_name).unwrap();
        let output = get_output(&mut solver, out_sat)?;
        if let Output::Verified = output {
            solver.assert(Bool::new_const(&actlit_name).neg())
                .map_err(|e| Error::SmtSolvingError(e.to_string()))?;
        }
        outputs.push((output, call_loc));
    }
    Ok(VerifyResult {
        solver_output: outputs,
        print_locs,
    })
}

pub fn check_execution(
    witness_map: WitnessMap<FieldElement>,
    circuit: &Circuit<FieldElement>,
    backend: BackendType,
    strict: bool,
    solver_options: &Vec<(String, String)>,
) -> Result<Output, Error> {
    let mut solver = create_solver(backend, false);
    for (key, value) in solver_options {
        solver.set_option(key, value);
    }

    let brillig_funcs: HashMap<u32, String> = HashMap::new();
    let use_int = use_int(backend);
    let next_witness_index = num_vars(circuit);
    let mut translator =
        Translator::new(&mut solver, brillig_funcs, next_witness_index, use_int, strict, Some(RangeOpts::default()));
    translator.translate_to_smt(circuit)?;
    if !translator.has_constraints() {
        println!("Circuit has no constraints");
    }
    translator.translate_witness_map(witness_map)?;

    let out_sat = solver.check_sat().map_err(|e| Error::SmtSolvingError(e.to_string()))?;
    let output = get_output(&mut solver, out_sat)?;
    Ok(output)
}
