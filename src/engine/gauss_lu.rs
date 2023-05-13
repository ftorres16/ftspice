pub fn solve(a_mat: &mut Vec<Vec<f64>>, b_vec: &mut Vec<f64>, x_vec: &mut Vec<f64>) {
    for curr_row in 0..a_mat.len() {
        // Pivot
        let mut max_idx = curr_row;
        let mut max_val = a_mat[curr_row][curr_row].abs();

        for next_row in curr_row + 1..a_mat.len() {
            let next_val = a_mat[next_row][curr_row].abs();

            if next_val > max_val {
                max_idx = next_row;
                max_val = next_val;
            }
        }

        if max_idx != curr_row {
            a_mat.swap(curr_row, max_idx);
            b_vec.swap(curr_row, max_idx);
        }

        // Scale
        for next_row in curr_row + 1..a_mat.len() {
            a_mat[next_row][curr_row] /= a_mat[curr_row][curr_row];
        }
        // Subtract
        for next_row in curr_row + 1..a_mat.len() {
            for next_col in curr_row + 1..a_mat.len() {
                a_mat[next_row][next_col] -= a_mat[next_row][curr_row] * a_mat[curr_row][next_col];
            }

            b_vec[next_row] -= a_mat[next_row][curr_row] * b_vec[curr_row];
        }
    }

    // Backwards substitution
    for row in 0..b_vec.len() {
        x_vec[row] = b_vec[row];
    }

    for curr_row in (0..a_mat.len()).rev() {
        x_vec[curr_row] /= a_mat[curr_row][curr_row];

        for next_row in (0..curr_row).rev() {
            x_vec[next_row] -= x_vec[curr_row] * a_mat[next_row][curr_row];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2x2_zero() {
        let mut a_mat = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let mut b_vec = vec![0.0, 0.0];
        let mut x_vec = vec![0.0; b_vec.len()];

        solve(&mut a_mat, &mut b_vec, &mut x_vec);

        assert_eq!(x_vec, [0.0, 0.0]);
    }

    #[test]
    fn test_2x2_nonzero() {
        let mut a_mat = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let mut b_vec = vec![1.0, 2.0];
        let mut x_vec = vec![0.0; b_vec.len()];

        solve(&mut a_mat, &mut b_vec, &mut x_vec);

        assert_eq!(x_vec, [1.0, 2.0]);
    }

    #[test]
    fn test_2x2_nontrivial() {
        let mut a_mat = vec![vec![5.0, 2.0], vec![-1.0, 3.0]];
        let mut b_vec = vec![1.0, 2.0];
        let mut x_vec = vec![0.0; b_vec.len()];

        solve(&mut a_mat, &mut b_vec, &mut x_vec);

        let eps = 1e-15;

        assert!((x_vec[0] - (-1.0 / 17.0)).abs() < eps);
        assert!((x_vec[1] - (11.0 / 17.0)).abs() < eps);
    }

    #[test]
    fn test_3x3_nontrivial() {
        let mut a_mat = vec![
            vec![5.0, 2.0, 1.0],
            vec![-1.0, 3.0, -1.0],
            vec![0.0, 2.0, -1.0],
        ];
        let mut b_vec = vec![1.0, 2.0, 1.0];
        let mut x_vec = vec![0.0; b_vec.len()];

        solve(&mut a_mat, &mut b_vec, &mut x_vec);

        let eps = 1e-15;

        assert!((x_vec[0] - (-2.0 / 9.0)).abs() < eps);
        assert!((x_vec[1] - (7.0 / 9.0)).abs() < eps);
        assert!((x_vec[2] - (5.0 / 9.0)).abs() < eps);
    }
}
