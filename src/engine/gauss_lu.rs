use ndarray::prelude::*;

pub fn solve(a_mat: &mut Array2<f64>, b_vec: &mut Array1<f64>, x_vec: &mut Array1<f64>) {
    for curr_row in 0..a_mat.nrows() {
        // Pivot
        let mut max_idx = curr_row;
        let mut max_val = a_mat[(curr_row, curr_row)].abs();

        for next_row in (curr_row + 1)..a_mat.nrows() {
            let next_val = a_mat[(next_row, curr_row)].abs();

            if next_val > max_val {
                max_idx = next_row;
                max_val = next_val;
            }
        }

        // Reorder rows
        if max_idx != curr_row {
            let mut it = a_mat.axis_iter_mut(Axis(0));
            ndarray::Zip::from(it.nth(curr_row).unwrap())
                .and(it.nth(max_idx - (curr_row + 1)).unwrap())
                .for_each(std::mem::swap);

            let tmp = b_vec[curr_row];
            b_vec[curr_row] = b_vec[max_idx];
            b_vec[max_idx] = tmp;
        }

        // Scale
        for next_row in (curr_row + 1)..a_mat.nrows() {
            a_mat[(next_row, curr_row)] /= a_mat[(curr_row, curr_row)];
        }
        // Subtract
        for next_row in (curr_row + 1)..a_mat.nrows() {
            for next_col in (curr_row + 1)..a_mat.nrows() {
                a_mat[(next_row, next_col)] -=
                    a_mat[(next_row, curr_row)] * a_mat[(curr_row, next_col)];
            }

            b_vec[next_row] -= a_mat[(next_row, curr_row)] * b_vec[curr_row];
        }
    }

    // Backwards substitution
    for row in 0..b_vec.len() {
        x_vec[row] = b_vec[row];
    }

    for curr_row in (0..a_mat.nrows()).rev() {
        x_vec[curr_row] /= a_mat[(curr_row, curr_row)];

        for next_row in (0..curr_row).rev() {
            x_vec[next_row] -= x_vec[curr_row] * a_mat[(next_row, curr_row)];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2x2_zero() {
        let mut a_mat = array![[1.0, 0.0], [0.0, 1.0]];
        let mut b_vec = array![0.0, 0.0];
        let mut x_vec = Array1::zeros(b_vec.len());

        solve(&mut a_mat, &mut b_vec, &mut x_vec);

        assert_eq!(x_vec, array![0.0, 0.0]);
    }

    #[test]
    fn test_2x2_nonzero() {
        let mut a_mat = array![[1.0, 0.0], [0.0, 1.0]];
        let mut b_vec = array![1.0, 2.0];
        let mut x_vec = Array1::zeros(b_vec.len());

        solve(&mut a_mat, &mut b_vec, &mut x_vec);

        assert_eq!(x_vec, array![1.0, 2.0]);
    }

    #[test]
    fn test_2x2_nontrivial() {
        let mut a_mat = array![[5.0, 2.0], [-1.0, 3.0]];
        let mut b_vec = array![1.0, 2.0];
        let mut x_vec = Array1::zeros(b_vec.len());

        solve(&mut a_mat, &mut b_vec, &mut x_vec);

        let eps = 1e-15;

        assert!((x_vec[0] - (-1.0 / 17.0)).abs() < eps);
        assert!((x_vec[1] - (11.0 / 17.0)).abs() < eps);
    }

    #[test]
    fn test_3x3_nontrivial() {
        let mut a_mat = array![[5.0, 2.0, 1.0], [-1.0, 3.0, -1.0], [0.0, 2.0, -1.0]];
        let mut b_vec = array![1.0, 2.0, 1.0];
        let mut x_vec = Array1::zeros(b_vec.len());

        solve(&mut a_mat, &mut b_vec, &mut x_vec);

        let eps = 1e-15;

        assert!((x_vec[0] - (-2.0 / 9.0)).abs() < eps);
        assert!((x_vec[1] - (7.0 / 9.0)).abs() < eps);
        assert!((x_vec[2] - (5.0 / 9.0)).abs() < eps);
    }
}
