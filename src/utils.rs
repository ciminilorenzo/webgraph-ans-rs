use std::ops::Neg;


pub fn entropy(distr: &[usize], total_freq: f64) -> f64 {
    let mut entropy = 0.0;

    for freq in distr {
        let pr = *freq as f64 / total_freq;
        entropy  += pr * f64::log2(pr);
    }
    entropy.neg()
}

/// Given the real probability distributions P and Q, calculates the cross entropy as follow:
/// ```text
/// cross-entropy(P|Q) = - ∑ p(x) * log(q(x))
/// ```
pub fn cross_entropy(distr: &[usize], m: f64, other_distr: &[usize], other_m: f64) -> f64 {
    assert_eq!(distr.len(), other_distr.len(), "Distr must have same length!");

    let mut cross_entropy = 0.0;

    for index in 0..distr.len() {
        if distr[index] == 0 { continue; }
        let p_x = distr[index] as f64 / m;
        let q_x = other_distr[index] as f64 / other_m;
        cross_entropy += p_x * f64::log2(q_x);
    }
    cross_entropy.neg()
}

pub fn self_entropy(distr: &[usize], m: f64) -> f64 {
    let mut self_entropy = 0.0;

    distr.iter().for_each(|freq| {
        if *freq == 0 { return; }

        self_entropy += *freq as f64 * f64::log2(m / *freq as f64);
    });
    self_entropy
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy() {
        let distr = [3_usize,3,4];
        assert_eq!("1.57", format!("{:.2}", entropy(&distr, 10_f64)));
    }

    #[test]
    fn test_distr_cross_entropy() {
        let distr = [3_usize,3,4];
        let other_distr = [4_usize,2,4];
        assert_eq!("1.62", format!("{:.2}", cross_entropy(&distr, 10_f64, &other_distr, 10_f64)));
    }

    #[test]
    fn test_self_entropy() {
        let distr = [3,3,2,1,1];
        let m = 10;

        assert_eq!("21.71", format!("{:.2}", self_entropy(&distr, m as f64)));
    }
}


