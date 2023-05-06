pub fn mat_vec_prod(a_mat: &Vec<Vec<f64>>, b_vec: &Vec<f64>) -> Vec<f64> {
    let mut out = vec![0.0; a_mat.len()];

    for i in 0..a_mat.len() {
        for j in 0..a_mat[0].len() {
            out[i] += a_mat[i][j] * b_vec[j];
        }
    }
    out
}

pub fn vec_add(a: &Vec<f64>, b: &Vec<f64>) -> Vec<f64> {
    a.iter().zip(b).map(|(x, y)| x + y).collect::<Vec<_>>()
}

pub fn vec_sub(a: &Vec<f64>, b: &Vec<f64>) -> Vec<f64> {
    a.iter().zip(b).map(|(x, y)| x - y).collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat_vec_prod_generic() {
        let a = vec![vec![1.0, 2.0], vec![2.0, 4.0], vec![-1.0, -1.0]];
        let b = vec![1.0, 2.0];
        let c = mat_vec_prod(&a, &b);

        assert_eq!(c, [5.0, 10.0, -3.0]);
    }

    #[test]
    fn vec_add_generic() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 3.0, 4.0];
        let c = vec_add(&a, &b);

        assert_eq!(c, [2.0, 5.0, 7.0]);
    }

    #[test]
    fn vec_sub_generic() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 3.0, 4.0];
        let c = vec_sub(&a, &b);

        assert_eq!(c, [0.0, -1.0, -1.0]);
    }
}
