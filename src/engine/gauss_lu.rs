use ndarray::prelude::*;

pub fn solve(a_mat: &mut Array2<f64>, b_vec: &mut Array1<f64>, x_vec: &mut Array1<f64>) {
    for k in 0..a_mat.nrows() {
        // Pivot
        let mut max_idx = k;
        let mut max_val = a_mat[(k, k)].abs();

        for next_row in (k + 1)..a_mat.nrows() {
            let next_val = a_mat[(next_row, k)].abs();

            if next_val > max_val {
                max_idx = next_row;
                max_val = next_val;
            }
        }

        // Reorder rows
        if max_idx != k {
            let mut it = a_mat.axis_iter_mut(Axis(0));
            ndarray::Zip::from(it.nth(k).unwrap())
                .and(it.nth(max_idx - (k + 1)).unwrap())
                .for_each(std::mem::swap);

            let tmp = b_vec[k];
            b_vec[k] = b_vec[max_idx];
            b_vec[max_idx] = tmp;
        }

        // Scale under diagonal
        let alpha = 1.0 / a_mat[(k, k)];
        a_mat.slice_mut(s![k + 1.., k]).mapv_inplace(|x| alpha * x);

        // Subtract
        for i in (k + 1)..a_mat.nrows() {
            for j in (k + 1)..a_mat.nrows() {
                a_mat[(i, j)] -= a_mat[(i, k)] * a_mat[(k, j)];
            }

            b_vec[i] -= a_mat[(i, k)] * b_vec[k];
        }
    }

    // Backwards substitution
    x_vec.assign(&b_vec);

    for i in (0..a_mat.nrows()).rev() {
        x_vec[i] /= a_mat[(i, i)];

        for j in (0..i).rev() {
            x_vec[j] -= x_vec[i] * a_mat[(j, i)];
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
