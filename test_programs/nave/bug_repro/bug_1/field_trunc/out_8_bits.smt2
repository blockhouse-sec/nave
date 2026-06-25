; Command:
; > cvc5 -q --no-interactive --lang smt2 --produce-models --incremental

(set-logic QF_FF)

(set-option :ff-solver gb)

( define-sort FF
   ( 
 
)
   (_ FiniteField 21888242871839275222246405745257275088548364400416034343698204186575808495617)
)

(declare-const w0 FF)

(declare-const w1 FF)

(declare-const w2 FF)

(declare-const w3 FF)

(declare-const w4 FF)

(declare-const w5 FF)

(declare-const w6 FF)

(assert
    (= (ff.add (ff.mul (as ff1 FF) w0) (ff.mul (as ff-1 FF) w2) (as ff-1 FF)) (as ff0 FF))
)

(declare-const w7 FF)

(assert
    (= (ff.mul w7 (ff.add w7 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w8 FF)

(assert
    (= (ff.mul w8 (ff.add w8 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w9 FF)

(assert
    (= (ff.mul w9 (ff.add w9 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w10 FF)

(assert
    (= (ff.mul w10 (ff.add w10 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w11 FF)

(assert
    (= (ff.mul w11 (ff.add w11 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w12 FF)

(assert
    (= (ff.mul w12 (ff.add w12 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w13 FF)

(assert
    (= (ff.mul w13 (ff.add w13 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w14 FF)

(assert
    (= (ff.mul w14 (ff.add w14 (as ff-1 FF))) (as ff0 FF))
)

(assert
    (= w2 (ff.bitsum w7 w8 w9 w10 w11 w12 w13 w14))
)

(assert
    (= (ff.add (ff.mul (as ff1 FF) w0) (as ff-127 FF)) (as ff0 FF))
)

(declare-const w15 FF)

(assert
    (= (ff.mul w15 (ff.add w15 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w16 FF)

(assert
    (= (ff.mul w16 (ff.add w16 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w17 FF)

(assert
    (= (ff.mul w17 (ff.add w17 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w18 FF)

(assert
    (= (ff.mul w18 (ff.add w18 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w19 FF)

(assert
    (= (ff.mul w19 (ff.add w19 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w20 FF)

(assert
    (= (ff.mul w20 (ff.add w20 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w21 FF)

(assert
    (= (ff.mul w21 (ff.add w21 (as ff-1 FF))) (as ff0 FF))
)

(declare-const w22 FF)

(assert
    (= (ff.mul w22 (ff.add w22 (as ff-1 FF))) (as ff0 FF))
)

(assert
    (= w3 (ff.bitsum w15 w16 w17 w18 w19 w20 w21 w22))
)

(declare-const w23 FF)

(assert
    (= (ff.mul w23 (ff.add w23 (as ff-1 FF))) (as ff0 FF))
)

(assert
    (= w4 w23)
)

(assert
    (= (ff.add (ff.mul (as ff1 FF) w2) (ff.mul (as ff-2 FF) w3) (ff.mul (as ff-1 FF) w4) (as ff256 FF)) (as ff0 FF))
)

(assert
    (= (ff.add (ff.mul (as ff1 FF) w4 w5) (ff.mul (as ff-25 FF) w5) (ff.mul (as ff1 FF) w6) (as ff-1 FF)) (as ff0 FF))
)

(assert
    (= (ff.add (ff.mul (as ff1 FF) w4 w6) (ff.mul (as ff-25 FF) w6) (as ff0 FF)) (as ff0 FF))
)

(declare-const actlit_0 Bool)

(declare-const w24 FF)

(assert
    (= (ff.add (ff.mul (as ff1 FF) w6) (as ff0 FF)) w24)
)

(assert
    (=> actlit_0 (= w24 (as ff0 FF)))
)

(assert
    (=> (not actlit_0) (= w24 (as ff1 FF)))
)

(assert
    (= (ff.add (ff.mul (as ff1 FF) w1) (ff.mul (as ff-1 FF) w4) (as ff0 FF)) (as ff0 FF))
)

(check-sat-assuming (
 
  actlit_0

))

(get-model)

