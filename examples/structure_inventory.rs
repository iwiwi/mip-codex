use mip_codex::mps;
use std::collections::HashSet;
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let m = mps::read(path).unwrap();
    let nnz: usize = m.rows.iter().map(|r| r.coefficients.len()).sum();
    let mut degrees = vec![0usize; m.variables.len()];
    let mut counts = [0usize; 4];
    let mut singleton_exact = 0;
    let mut duplicate = 0;
    let mut seen = HashSet::new();
    for row in &m.rows {
        let n = row.coefficients.len();
        counts[n.min(3)] += 1;
        for &(j, _) in &row.coefficients {
            degrees[j] += 1;
        }
        if n == 1 && row.exact_coefficients {
            singleton_exact += 1;
        }
        if nnz <= 2_000_000 && row.exact_coefficients {
            let key: Vec<_> = row
                .coefficients
                .iter()
                .map(|&(j, a)| (j, a.to_bits()))
                .collect();
            if !seen.insert(key) {
                duplicate += 1;
            }
        }
    }
    println!(
        "{}",
        serde_json::json!({"variables":m.variables.len(),"rows":m.rows.len(),"nnz":nnz,"row_lengths_0_1_2_3plus":counts,"exact_singletons":singleton_exact,"exact_duplicate_coefficients":if nnz<=2_000_000{Some(duplicate)}else{None},"degree0":degrees.iter().filter(|&&n|n==0).count(),"degree1_continuous":m.variables.iter().zip(&degrees).filter(|(v,n)|!v.integer&&**n==1).count(),"degree1_integer":m.variables.iter().zip(&degrees).filter(|(v,n)|v.integer&&**n==1).count(),"equalities":m.rows.iter().filter(|r|r.lower.is_some()&&r.lower==r.upper).count(),"fixed":m.variables.iter().filter(|v|v.lower.is_some()&&v.lower==v.upper).count()})
    );
}
