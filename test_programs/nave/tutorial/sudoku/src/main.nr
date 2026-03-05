//! Sudoku verifier

// Variant 1: 4*4 ------------------------
fn main(board: [[Field; 4]; 4]) -> pub bool {
    // Example board
    // let board: [[u32; 4]; 4] = [
    //     [1, 2, 3, 4],
    //     [3, 4, 1, 2],
    //     [2, 1, 4, 3],
    //     [4, 3, 2, 1],
    // ];
    let solution = verify_sudoku(board);
    unsafe { verify_assert(solution); }
    solution
}

fn verify_sudoku(board: [[Field; 4]; 4]) -> bool {
    // Valid values
    for i in 0..4 {
        for j in 0..4 {
            let val = board[i][j];
            assert((val == 1) | (val == 2) | (val == 3) | (val == 4));
        }
    }

    let mut row_values: [Field; 4] = [0; 4];
    let mut col_values: [Field; 4] = [0; 4];

    // Columns
    for c in 0..4 {
        let mut seen = [false; 4];
        for r in 0..4 {
            let index = (board[r][c] - 1) as u32;
            assert(!seen[index]);
            seen[index] = true;
        }
    }

    // Rows
    for r in 0..4 {
        let mut seen = [false; 4];
        for c in 0..4 {
            let index = (board[r][c] - 1) as u32;
            assert(!seen[index]);
            seen[index] = true;
        }
    }

    // 2x2 boxes
    for br in 0..2 {
        for bc in 0..2 {
            let box_r = br * 2;
            let box_c = bc * 2;

            let mut seen = [false; 4];

            for r in box_r..box_r + 2 {
                for c in box_c..box_c + 2 {
                    let index = (board[r][c] - 1) as u32;
                    assert(!seen[index]);
                    seen[index] = true;
                }
            }
        }
    }
    true
}

unconstrained fn verify_assert(b: bool) { assert(false); }

// Variant 2: 4 * 4 ----------------
// Generate more constraints than the previous one
// Reference: https://paragraph.com/@sebagiraudo/implementing-a-zk-sudoku-validator-in-noir
// fn main(puzzle: pub [Field; 16], solution: [Field; 16]) {
//     for i in 0..16 {
//         if puzzle[i] != 0 {
//             assert (puzzle[i] == solution[i]);
//             assert(solution[i] as u8 >= 1);
//             assert(solution[i] as u8 <= 4);
//         }
//     }
//
//     // rows:
//     let mut row_values: [Field; 4] = [0; 4];
//     for row in 0..4 {
//         row_values = [0; 4];
//         for col in 0..4 {
//             row_values[col] = solution[row * 4 + col];
//         }
//         unsafe { verify_assert(check_all_unique(row_values)); }
//     }
//
//     // columns:
//     let mut col_values: [Field; 4] = [0; 4];
//     for col in 0..4 {
//         col_values = [0; 4];
//         for row in 0..4 {
//             col_values[row] = solution[row * 4 + col];
//         }
//         unsafe { verify_assert(check_all_unique(col_values)); }
//     }
//
//     // 2x2 subgrids
//     let mut box_values: [Field; 4] = [0; 4];
//     for box_row in [0, 2] {
//         for box_col in [0, 2] {
//             box_values = [0; 4];
//             let mut index = 0;
//             for i in 0..2 {
//                 for j in 0..2 {
//                     box_values[index] = solution[(box_row + i) * 4 + (box_col + j)];
//                     index += 1;
//                 }
//             }
//             unsafe { verify_assert(check_all_unique(box_values)); }
//         }
//     }
// }

// fn check_all_unique(numbers: [Field; 4]) -> bool {
//     let mut all_unique: bool = true;
//     for i in 0..4 {
//         for j in (i + 1)..4 {
//             if numbers[i] == numbers[j] {
//                 all_unique = false;
//             }
//         }
//     }
//     all_unique
// }

// Variant 3: 2*2 --------------------------------------------
// fn main(board: [[Field; 2]; 2]) {
//     unsafe { verify_assert(is_valid(board)); }
// }

// fn valid_pair(a: Field, b: Field) -> bool {
//     ((a == 1) | (a == 2)) & ((b == 1) | (b == 2)) & (a != b)
// }

// fn is_valid(board: [[Field; 2]; 2]) -> bool {
//     // rows
//     assert(valid_pair(board[0][0], board[0][1]));
//     assert(valid_pair(board[1][0], board[1][1]));
//
//     // columns
//     assert(valid_pair(board[0][0], board[1][0]));
//     assert(valid_pair(board[0][1], board[1][1]));
//
//     true
// }