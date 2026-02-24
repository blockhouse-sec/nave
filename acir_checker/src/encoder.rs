// Encoder for translating ACIR to SMT

use std::{
    collections::HashMap, marker::PhantomData, str::FromStr, u32
};
use acir::{
    AcirField, circuit::{
        Circuit,
        brillig::{
            BrilligFunctionId, 
            BrilligInputs, 
            BrilligOutputs
        },
        opcodes::{
            AcirFunctionId, 
            BlackBoxFuncCall, 
            BlockId, 
            BlockType,
            FunctionInput, 
            MemOp,
        },
    }, native_types::{
        Expression, 
        Witness, WitnessMap,
    }
};
use num_bigint::{BigInt, BigUint};

use crate::{
    error::Error,
    smt::{
        Bool, 
        FField, 
        Int, 
        Solver, 
        Type,
    }
};

pub(crate) struct Translator<'a, F: AcirField> {
    solver: &'a mut Solver,
    brillig_funcs: HashMap<u32, String>,
    next_witness_index: u32,
    use_int: bool,
    strict: bool,
    // The first element of the tuple specifies the name of the conditional literal
    // that holds the result of the verification assert. The second element
    // specifies the location of the Brillig call opcode corresponding to the
    // verification assert.
    ver_conds: Vec<(String, usize)>,
    _f: PhantomData<F>,
}

struct MemTrace<F: AcirField> {
    block_id: BlockId,
    block_type: BlockType,
    init: Vec<Witness>,
    ops: Vec<MemOp<F>>,
}

impl<'a, F: AcirField> Translator<'a, F> {
    pub(crate) fn new(
        solver: &'a mut Solver,
        brillig_funcs: HashMap<u32, String>,
        next_witness_index: u32,
        use_int: bool,
        strict: bool,
    ) -> Translator<'a, F> {
        assert!(solver.prime() == F::modulus().to_string());
        Translator {
            solver,
            brillig_funcs,
            next_witness_index,
            use_int,
            strict,
            ver_conds: Vec::new(),
            _f: PhantomData,
        }
    }

    // Assumes that witnesses in witness_map are already declared in the solver
    pub(crate) fn translate_witness_map(&mut self, witness_map: WitnessMap<F>) -> Result<(), Error> {
        println!("Translating witness map {:?}", witness_map);
        for (witness, value) in witness_map {
            if self.use_int {
                let wit = self.new_const_int(witness);
                let val_big_uint = BigInt::from_str(&value.to_string()).unwrap();
                let val = if val_big_uint >= BigInt::ZERO && val_big_uint < F::modulus().into() {
                    self.new_element_int(value)
                } else {
                    self.new_element_int(value).modu(self.prime_int())
                };
                self.solver.assert(wit.eq(val));
            } else {
                let wit = self.new_const(witness);
                let val = self.new_element(value);
                self.solver.assert(wit.eq(val));
            }
        }
        Ok(())
    }

    pub(crate) fn translate_to_smt(
        &mut self, 
        circuit: &Circuit<F>
    ) -> Result<(), Error> {
        let num_vars = num_vars(circuit);
        let _witnesses = circuit.circuit_arguments();
        let _public_inputs = circuit.public_inputs();
        
        for wi in 0..num_vars {
            if self.use_int {
                self.solver.declare_const(&witness_name(Witness(wi)), Type::Int);
                self.encode_int_range(Witness(wi));
            } else {
                self.solver.declare_const(&witness_name(Witness(wi)), Type::FField);
            }
        }
        let mut mem_traces: HashMap<BlockId, MemTrace<F>> = HashMap::new();
        for (opcode_loc, opcode) in circuit.opcodes.iter().enumerate() {
            match opcode {
                acir::circuit::Opcode::AssertZero(expression) => {
                    self.translate_assert_zero(expression);
                }
                acir::circuit::Opcode::BlackBoxFuncCall(black_box_func_call) => {
                    let res = self.translate_blackbox_call(black_box_func_call);
                    check_strictness(self.strict, res)?;
                }
                acir::circuit::Opcode::MemoryOp { block_id, op } => {
                    let mem_trace = mem_traces
                        .get_mut(block_id)
                        .expect("MemInit opcode should have run before");
                    mem_trace.ops.push(op.clone());
                }
                acir::circuit::Opcode::MemoryInit { block_id, init, block_type } => {
                    let mem_trace = MemTrace {
                        block_id: *block_id,
                        block_type: block_type.clone(),
                        init: init.iter().cloned().collect(),
                        ops: Vec::new(),
                    };
                    mem_traces.insert(*block_id, mem_trace);
                }
                acir::circuit::Opcode::BrilligCall { id, inputs, outputs, predicate } => {
                    let res = self.translate_brilling_call(opcode_loc, *id, inputs, outputs, predicate);
                    check_strictness(self.strict, res)?;
                }
                acir::circuit::Opcode::Call { id, inputs, outputs, predicate } => {
                    let res = self.translate_call(*id, inputs, outputs, predicate);
                    check_strictness(self.strict, res)?;
                }
            }
        }
        for mem_trace in mem_traces.values() {
            let res = self.translate_memory_init(
                mem_trace.block_id,
                &mem_trace.init,
                &mem_trace.block_type,
            );
            check_strictness(self.strict, res)?;
            let mem_block_len = mem_trace.init.len();
            let mut f = 0;
            for op in &mem_trace.ops {
                f = if self.use_int {
                    self.translate_memory_op_int(mem_trace.block_id, op, mem_block_len, f)
                } else {
                    self.translate_memory_op(mem_trace.block_id, op, mem_block_len, f)
                };
            }
        }
        Ok(())
    }

    fn translate_memory_init(
        &mut self,
        block_id: BlockId,
        init: &[Witness],
        block_type: &BlockType,
    ) -> Result<(), Error> {
        match block_type {
            BlockType::Memory => {}
            _ => {
                return Err(Error::EncodingError(
                    "Only memory block type is supported".to_string(),
                ));
            }
        }
        for (i, wit) in init.iter().enumerate() {
            self.solver.declare_const(
                &format!("_m_{}_{}_0", block_id.0, i),
                if self.use_int { Type::Int } else { Type::FField },
            );
            let gate = FField::new_const(&format!("_m_{}_{}_0", block_id.0, i));
            let wit = self.new_const(*wit);
            let eq = wit.eq(gate);
            self.solver.assert(eq);
        }
        Ok(())
    }

    // Predicate is excluded
    fn translate_memory_op(
        &mut self, 
        block_id: BlockId, 
        op: &MemOp<F>, 
        len: usize, 
        f: u32
    ) -> u32 {
        let value = &op.value;
        let index = &op.index;
        let op = &op.operation;
        assert!(op.is_const());

        let index_witness = self.cur_new_witness();
        let index_wit = self.new_witness();
        let index_exp = self.translate_expression(index);
        self.solver.assert(index_wit.clone().eq(index_exp));
        // Ensure index is in range
        self.translate_range_inclusive(index_witness, len as u32);

        let value_wit = self.new_witness();
        let value_exp = self.translate_expression(value);
        self.solver.assert(value_wit.clone().eq(value_exp));

        if op.is_zero() {
            // Read operation
            // index != op.value == x0
            for i in 0..len {
                let indexed_mem_value =
                    FField::new_const(&format!("_m_{}_{}_{}", block_id.0, i, f));
                let index_value = FField::new_value(&format!("{}", i));
                let exp =
                    index_wit.clone().eq(index_value).imp(value_wit.clone().eq(indexed_mem_value));
                self.solver.assert(exp);
            }
            f
        } else {
            // Write operation
            assert!(op == &Expression::one());
            let f_new = f + 1;
            for i in 0..len {
                self.solver
                    .declare_const(&format!("_m_{}_{}_{}", block_id.0, i, f_new), Type::FField);
                let indexed_mem_value =
                    FField::new_const(&format!("_m_{}_{}_{}", block_id.0, i, f_new));
                let pre_indexed_mem_value =
                    FField::new_const(&format!("_m_{}_{}_{}", block_id.0, i, f));

                let index_value = FField::new_value(&format!("{}", i));
                let value_change_exp = index_wit
                    .clone()
                    .eq(index_value.clone())
                    .imp(value_wit.clone().eq(indexed_mem_value.clone()));
                self.solver.assert(value_change_exp);
                let other_values_remain_exp = index_wit
                    .clone()
                    .eq(index_value)
                    .neg()
                    .imp(pre_indexed_mem_value.eq(indexed_mem_value));
                self.solver.assert(other_values_remain_exp);
            }
            f_new
        }
    }

    fn translate_memory_op_int(
        &mut self,
        block_id: BlockId,
        op: &MemOp<F>,
        len: usize,
        f: u32,
    ) -> u32 {
        let value = &op.value;
        let index = &op.index;
        let op = &op.operation;
        assert!(op.is_const());

        let index_witness = self.cur_new_witness();
        let index_wit = self.new_witness_int();
        let index_exp = self.translate_expression_int(index);
        self.solver.assert(index_wit.clone().eq(index_exp));
        // Ensure index is in range
        self.translate_range_inclusive_int(index_witness, len as u32);

        let value_wit = self.new_witness_int();
        let value_exp = self.translate_expression_int(value);
        self.solver.assert(value_wit.clone().eq(value_exp));

        if op.is_zero() {
            // Read operation
            // index != op.value == x0
            for i in 0..len {
                let indexed_mem_value = Int::new_const(&format!("_m_{}_{}_{}", block_id.0, i, f));
                let index_value = Int::new_value(&format!("{}", i));
                let exp =
                    index_wit.clone().eq(index_value).imp(value_wit.clone().eq(indexed_mem_value));
                self.solver.assert(exp);
            }
            f
        } else {
            // Write operation
            assert!(op == &Expression::one());
            let f_new = f + 1;
            for i in 0..len {
                self.solver
                    .declare_const(&format!("_m_{}_{}_{}", block_id.0, i, f_new), Type::Int);
                let indexed_mem_value =
                    Int::new_const(&format!("_m_{}_{}_{}", block_id.0, i, f_new));
                let pre_indexed_mem_value =
                    Int::new_const(&format!("_m_{}_{}_{}", block_id.0, i, f));
                let index_value = Int::new_value(&format!("{}", i));
                let value_change_exp = index_wit
                    .clone()
                    .eq(index_value.clone())
                    .imp(value_wit.clone().eq(indexed_mem_value.clone()));
                self.solver.assert(value_change_exp);
                let other_values_remain_exp = index_wit
                    .clone()
                    .eq(index_value)
                    .neg()
                    .imp(pre_indexed_mem_value.eq(indexed_mem_value));
                self.solver.assert(other_values_remain_exp);
            }
            f_new
        }
    }

    fn translate_blackbox_call(
        &mut self,
        black_box_func_call: &BlackBoxFuncCall<F>,
    ) -> Result<(), Error> {
        match black_box_func_call {
            BlackBoxFuncCall::AND { .. } => {
                Err(Error::EncodingError("AND black box function is not supported".to_string()))
            }
            BlackBoxFuncCall::XOR { .. } => {
                Err(Error::EncodingError("XOR black box function is not supported".to_string()))
            }
            BlackBoxFuncCall::RANGE { input, num_bits } => {
                self.translate_range(input, *num_bits);
                Ok(())
            }
            BlackBoxFuncCall::AES128Encrypt { .. } => Err(Error::EncodingError(
                "AES128Encrypt black box function is not supported".to_string(),
            )),
            BlackBoxFuncCall::Blake2s { .. } => {
                Err(Error::EncodingError("Blake2s black box function is not supported".to_string()))
            }
            BlackBoxFuncCall::Blake3 { .. } => {
                Err(Error::EncodingError("Blake3 black box function is not supported".to_string()))
            }
            BlackBoxFuncCall::EcdsaSecp256k1 { .. } => Err(Error::EncodingError(
                "EcdsaSecp256k1 black box function is not supported".to_string(),
            )),
            BlackBoxFuncCall::EcdsaSecp256r1 { .. } => Err(Error::EncodingError(
                "EcdsaSecp256r1 black box function is not supported".to_string(),
            )),
            BlackBoxFuncCall::MultiScalarMul { .. } => Err(Error::EncodingError(
                "MultiScalarMul black box function is not supported".to_string(),
            )),
            BlackBoxFuncCall::EmbeddedCurveAdd { .. } => Err(Error::EncodingError(
                "EmbeddedCurveAdd black box function is not supported".to_string(),
            )),
            BlackBoxFuncCall::Keccakf1600 { .. } => Err(Error::EncodingError(
                "Keccakf1600 black box function is not supported".to_string(),
            )),
            BlackBoxFuncCall::RecursiveAggregation { .. } => Err(Error::EncodingError(
                "RecursiveAggregation black box function is not supported".to_string(),
            )),
            BlackBoxFuncCall::Poseidon2Permutation { .. } => Err(Error::EncodingError(
                "Poseidon2Permutation black box function is not supported".to_string(),
            )),
            BlackBoxFuncCall::Sha256Compression { .. } => Err(Error::EncodingError(
                "Sha256Compression black box function is not supported".to_string(),
            )),
        }
    }

    fn translate_assert_zero(&mut self, expression: &Expression<F>) {
        if self.use_int {
            let exp = self.translate_expression_int(expression);
            let zero = self.zero_int();
            self.solver.assert(exp.eq(zero));
        } else {
            let exp = self.translate_expression(expression);
            let zero = self.zero();
            self.solver.assert(exp.eq(zero));
        }
    }

    // fn translate_assert_one(&mut self, expression: &Expression<F>) {
    //     if self.use_int {
    //         let exp = self.translate_expression_int(expression);
    //         let one = self.one_int();
    //         self.solver.assert(exp.eq(one));
    //     } else {
    //         let exp = self.translate_expression(expression);
    //         let one = self.one();
    //         self.solver.assert(exp.eq(one));
    //     }
    // }

    fn translate_expression(&mut self, expression: &Expression<F>) -> FField {
        let mut exps = Vec::new();
        for mul_term in &expression.mul_terms {
            let element = self.new_element(mul_term.0);
            let wit0 = self.new_const(mul_term.1);
            let wit1 = self.new_const(mul_term.2);
            exps.push(FField::rmul(vec![element, wit0, wit1]));
        }
        for lin_term in &expression.linear_combinations {
            let element = self.new_element(lin_term.0);
            let wit = self.new_const(lin_term.1);
            exps.push(FField::rmul(vec![element, wit]));
        }
        let element = FField::new_value(&expression.q_c.to_string());
        if exps.is_empty() {
            element
        } else {
            exps.push(element);
            FField::radd(exps)
        }
    }

    fn translate_expression_int(&mut self, expression: &Expression<F>) -> Int {
        let mut exps = Vec::new();
        for mul_term in &expression.mul_terms {
            let element = self.new_element_int(mul_term.0);
            let wit0 = self.new_const_int(mul_term.1);
            let wit1 = self.new_const_int(mul_term.2);
            exps.push(Int::rmul(vec![element, wit0, wit1]));
        }
        for lin_term in &expression.linear_combinations {
            let element = self.new_element_int(lin_term.0);
            let wit = self.new_const_int(lin_term.1);
            exps.push(Int::rmul(vec![element, wit]));
        }
        let element = Int::new_value(&expression.q_c.to_string());
        if exps.is_empty() {
            element
        } else {
            exps.push(element);
            let prime = self.prime_int();
            Int::modu(Int::radd(exps), prime)
        }
    }

    fn translate_brilling_call(
        &mut self,
        opcode_loc: usize,
        id: BrilligFunctionId,
        inputs: &[BrilligInputs<F>],
        _outputs: &[BrilligOutputs],
        predicate: &Expression<F>,
    ) -> Result<(), Error> {
        // predicate indicates if brillig call should be skipped
        if let Some(func_name) = self.brillig_funcs.get(&id.0) {
            return match func_name.as_str() {
                ASSERT_FUNC_NAME => self.translate_verify_assert(opcode_loc, inputs, predicate),
                // PRECONDITION_FUNC_NAME => self.translate_verify_precondition(inputs),
                // POSTCONDITION_FUNC_NAME => self.translate_verify_postcondition(inputs),
                ASSUME_FUNC_NAME => self.translate_verify_assume(opcode_loc, inputs, predicate),
                _ => Ok(())
            }
        }
        Ok(())
    }

    fn translate_verify_assert(
        &mut self, 
        opcode_loc: usize,
        inputs: &[BrilligInputs<F>],
        predicate: &Expression<F>,
    ) -> Result<(), Error> {
        for input in inputs {
            match input {
                BrilligInputs::Single(exp) => {
                    let new_cond_lit = self.new_cond_lit(opcode_loc);
                    if self.use_int {
                        // match predicate {
                        //     Some(pred_exp) => {
                        //         let exp_int = self.translate_expression_int(pred_exp);
                        //         let one = self.one_int();
                        //         self.solver.assert(exp_int.eq(one));
                        //     },
                        //     None => {}
                        // }
                        // TODO: properly handle predicate instead of assuming it's always one
                        assert!(predicate == &Expression::one());

                        let exp_int = self.translate_expression_int(exp);
                        let new_wit = self.new_witness_int();
                        self.solver.assert(exp_int.eq(new_wit.clone()));
                        let zero = self.zero_int();
                        let one = self.one_int();
                        self.solver.assert(new_cond_lit.clone().imp(new_wit.clone().eq(zero)));
                        self.solver.assert(new_cond_lit.neg().imp(new_wit.clone().eq(one)));
                    } else {
                        // match predicate {
                        //     Some(pred_exp) => {
                        //         let exp = self.translate_expression(pred_exp);
                        //         let one = self.one();
                        //         self.solver.assert(exp.eq(one));
                        //     },
                        //     None => {}
                        // }

                        // TODO: properly handle predicate instead of assuming it's always one
                        assert!(predicate == &Expression::one());



                        let exp = self.translate_expression(exp);
                        let new_wit = self.new_witness();
                        self.solver.assert(exp.eq(new_wit.clone()));
                        let zero = self.zero();
                        let one = self.one();
                        self.solver.assert(new_cond_lit.clone().imp(new_wit.clone().eq(zero)));
                        self.solver.assert(new_cond_lit.neg().imp(new_wit.clone().eq(one)));
                    }
                }
                BrilligInputs::Array(_exps) => {
                    return Err(Error::EncodingError(
                        "Brillig input array is unimplemented".to_string(),
                    ));
                }
                BrilligInputs::MemoryArray(_id) => {
                    return Err(Error::EncodingError(
                        "Brillig input memory array is unimplemented".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn translate_verify_assume(
        &mut self,
        _opcode_loc: usize,
        inputs: &[BrilligInputs<F>],
        predicate: &Expression<F>,
    ) -> Result<(), Error> {
        for input in inputs {
            match input {
                BrilligInputs::Single(exp) => {
                    if self.use_int {
                        let one_int = self.one_int();
                        // match predicate {
                        //     Some(pred_exp) => {
                        //         let exp_int = self.translate_expression_int(pred_exp);
                        //         self.solver.assert(exp_int.eq(one_int.clone()));
                        //     },
                        //     None => {}
                        // }

                        // TODO: properly handle predicate instead of assuming it's always one
                        assert!(predicate == &Expression::one());

                        let exp_int = self.translate_expression_int(exp);
                        self.solver.assert(exp_int.eq(one_int));
                    } else {
                        let one = self.one();
                        // match predicate {
                        //     Some(pred_exp) => {
                        //         let exp = self.translate_expression(pred_exp);
                        //         self.solver.assert(exp.eq(one.clone()));
                        //     },
                        //     None => {}
                        // }

                        // TODO: properly handle predicate instead of assuming it's always one
                        assert!(predicate == &Expression::one());

                        let exp = self.translate_expression(exp);
                        self.solver.assert(exp.eq(one));
                    }
                }
                BrilligInputs::Array(_exps) => {
                    return Err(Error::EncodingError(
                        "Brillig input array is unimplemented".to_string(),
                    ));
                }
                BrilligInputs::MemoryArray(_id) => {
                    return Err(Error::EncodingError(
                        "Brillig input memory array is unimplemented".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    // fn translate_verify_precondition(&mut self, inputs: &[BrilligInputs<F>]) {
    //     for input in inputs {
    //         match input {
    //             BrilligInputs::Single(exp) => {
    //                 let _ = self.translate_assert_one(exp);
    //             }
    //             _ => {
    //                 unimplemented!("assume should be simple expression")
    //             }
    //         }
    //     }
    // }

    // fn translate_verify_postcondition(&mut self, inputs: &[BrilligInputs<F>]) {
    //     for input in inputs {
    //         match input {
    //             BrilligInputs::Single(exp) => {
    //                 let _ = self.translate_assert_zero(exp);
    //             }
    //             _ => {
    //                 unimplemented!("assume should be simple expression")
    //             }
    //         }
    //     }
    // }

    fn translate_call(
        &self,
        _id: AcirFunctionId,
        _inputs: &[Witness],
        _outputs: &[Witness],
        _predicate: &Expression<F>,
    ) -> Result<(), Error> {
        {
            return Err(Error::EncodingError(
                "Function calls are not supported in formal verification".to_string(),
            ));
        }
    }

    fn translate_range(&mut self, input: &FunctionInput<F> , num_bits: u32) {
        // TODO: optimise to combine all ranges over the same variable
        match input {
            FunctionInput::Constant(_) => {
                println!("unimplemented constant input");
                return;
            }
            FunctionInput::Witness(witness) => {
                if self.use_int {
                    self.translate_range_int(*witness, num_bits);
                } else {
                    self.translate_range_bitsum(*witness, num_bits);
                }
            }
        }
    
    }

    fn translate_range_int(&mut self, witness: Witness, num_bits: u32) {
        if num_bits == 0 {
            let wit = self.new_const_int(witness);
            let zero = self.zero_int();
            self.solver.assert(wit.eq(zero));
            return;
        }
        let wit = self.new_const_int(witness);
        let minus_one = Self::minus_one_int();
        self.solver.assert(minus_one.lt(wit.clone()));
        let val_int = BigUint::from(2u32).pow(num_bits.into());
        let value = self.new_element_big_int(val_int);
        self.solver.assert(wit.lt(value));
    }

    fn translate_range_bitsum(&mut self, witness: Witness, num_bits: u32) {
        if num_bits == 0 {
            let wit = self.new_const(witness);
            let zero = self.zero();
            self.solver.assert(wit.eq(zero));
            return;
        }
        self.encode_bitsum(witness, num_bits as usize);
    }

    // Create a range constraint witness in [0, length-1]
    fn translate_range_inclusive(&mut self, witness: Witness, length: u32) {
        assert!(length > 0);
        let bound = length - 1;
        if bound == 0 {
            let wit = self.new_const(witness);
            let zero = self.zero();
            self.solver.assert(wit.eq(zero));
            return;
        }
        let num_bits = bound.ilog2() + 1;
        let bitvec= self.encode_bitsum(witness, num_bits as usize);

        // This loop enforces that the bit representation is less than or equal to bound
        let mut cur_ones_bit_field = bitvec[(num_bits - 1) as usize].clone();
        for i in (0..num_bits-1).rev() {
            let bit_i = (bound >> i) & 1;
            if bit_i == 1 {
                let new_ones_bit_field = self.new_witness();
                self.solver.assert(new_ones_bit_field.clone().eq(cur_ones_bit_field.clone().mul(bitvec[i as usize].clone())));
                cur_ones_bit_field = new_ones_bit_field;
            } else {
                let bit_i_field = bitvec[i as usize].clone();
                let zero = self.zero();
                self.solver.assert(bit_i_field.mul(cur_ones_bit_field.clone()).eq(zero));
            }
        }
    }

    fn translate_range_inclusive_int(&mut self, witness: Witness, length: u32) {
        assert!(length > 0);
        let bound = length - 1;
        if bound == 0 {
            let wit = self.new_const_int(witness);
            let zero = self.zero_int();
            self.solver.assert(wit.eq(zero));
            return;
        }
        let wit = self.new_const_int(witness);
        let minus_one = Self::minus_one_int();
        self.solver.assert(minus_one.lt(wit.clone()));
        let value = self.new_element_int(F::from(length));
        self.solver.assert(wit.lt(value));
    }

    // This encode the expression witness = \sum^{num_bits}_{i=0} 2^i * out[i];
    // where out[i] is a field element in \{0,1}.
    fn encode_bitsum(&mut self, witness: Witness, num_bits: usize) -> Vec<FField> {
        // assert!(num_bits > 0);
        let mut res = Vec::with_capacity(num_bits);
        for _ in 0..num_bits {
            let out_i = self.encode_bool();
            res.push(out_i);
        }
        let exp = if num_bits == 1 { res[0].clone() } else { FField::rbitsum(res.clone()) };
        let wit = self.new_const(witness);
        self.solver.assert(wit.eq(exp));
        res
    }

    // Create a field constant out such that out in \{0,1}.
    fn encode_bool(&mut self) -> FField {
        let res = self.new_witness();
        let zero = FField::zero();
        self.solver.assert(res.clone().mul(res.clone().add(Self::minus_one())).eq(zero));
        res
    }

    fn encode_int_range(&mut self, witness: Witness) {
        let wit = self.new_const_int(witness);
        let minus_one = Self::minus_one_int();
        self.solver.assert(minus_one.lt(wit.clone()));
        let prime_int = self.prime_int();
        self.solver.assert(wit.lt(prime_int));
    }

    fn new_const(&mut self, witness: Witness) -> FField {
        let wit_name = witness_name(witness);
        FField::new_const(&wit_name)
    }

    fn new_const_int(&mut self, witness: Witness) -> Int {
        let wit_name = witness_name(witness);
        Int::new_const(&wit_name)
    }

    fn prime_int(&self) -> Int {
        Int::new_value(self.solver.prime())
    }

    fn new_element(&mut self, element: F) -> FField {
        let element_value = element_value(element);
        FField::new_value(&element_value)
    }

    fn new_element_int(&mut self, element: F) -> Int {
        let element_value = element_value(element);
        Int::new_value(&element_value)
    }

    fn new_element_big_int(&mut self, element: BigUint) -> Int {
        Int::new_value(&element.to_string())
    }

    fn zero(&mut self) -> FField {
        FField::zero()
    }

    fn zero_int(&mut self) -> Int {
        Int::zero()
    }

    fn one(&mut self) -> FField {
        FField::one()
    }

    fn one_int(&mut self) -> Int {
        Int::one()
    }

    fn minus_one() -> FField {
        FField::new_value("-1")
    }

    fn minus_one_int() -> Int {
        Int::new_value("-1")
    }

    fn cur_new_witness(&mut self) -> Witness {
        Witness(self.next_witness_index)
    }

    fn new_witness(&mut self) -> FField {
        let new_wit = Witness(self.next_witness_index);
        let new_wit_name = witness_name(new_wit);
        self.next_witness_index += 1;
        self.solver.declare_const(&new_wit_name, Type::FField);
        FField::new_const(&new_wit_name)
    }

    fn new_witness_int(&mut self) -> Int {
        let new_wit = Witness(self.next_witness_index);
        let new_wit_name = witness_name(new_wit);
        self.next_witness_index += 1;
        self.solver.declare_const(&new_wit_name, Type::Int);
        self.encode_int_range(new_wit);
        Int::new_const(&new_wit_name)
    }

    fn new_cond_lit(&mut self, opcode_loc: usize) -> Bool {
        let new_cond_lit_index = self.ver_conds.len() as u32;
        let new_cond_lit_name = actlit_name(new_cond_lit_index);
        self.solver.declare_const(&new_cond_lit_name, Type::Bool);
        let new_cond_lit = Bool::new_const(&new_cond_lit_name);
        self.ver_conds.push((new_cond_lit_name, opcode_loc));
        new_cond_lit
    }

    pub(crate) fn ver_conds(&self) -> Vec<(String, usize)> {
        self.ver_conds.clone()
    }
}

pub(crate) fn num_vars<F: AcirField>(circuit: &Circuit<F>) -> u32 {
    circuit.current_witness_index + 1
}

fn check_strictness(strict: bool, res: Result<(), Error>) -> Result<(), Error> {
    if strict {
        res
    } else {
        match res {
            Ok(()) => Ok(()),
            Err(_) => Ok(()),
        }
    }
}

fn actlit_name(index: u32) -> String {
    format!("actlit_{}", index)
}

fn witness_name(wit: Witness) -> String {
    format!("_{}", wit.0)
}

fn element_value<F: AcirField>(element: F) -> String {
    element.to_string()
}

const ASSERT_FUNC_NAME: &str = "verify_assert";

const ASSUME_FUNC_NAME: &str = "verify_assume";

// const PRECONDITION_FUNC_NAME: &str = "verify_pre";

// const POSTCONDITION_FUNC_NAME: &str = "verify_post";

