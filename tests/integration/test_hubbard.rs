/// Integration tests for Hubbard model Hamiltonian processing.
///
/// These tests verify the full pipeline (parse → expand → normal order → combine → output)
/// against known analytical results.

// TODO: uncomment once crate is implemented
// use quantum_simpl::{parse, pipeline, output};

/// Test 1: 2-site Hubbard model produces correct number of terms.
///
/// H = -t Σ_σ (c†_{0,σ} c_{1,σ} + h.c.) + U Σ_i n_{i,↑} n_{i,↓}
///
/// After expansion: 4 hopping terms + 2 interaction terms = 6 terms
/// After normal ordering of n_↑ n_↓: each becomes c†c†cc (already normal ordered)
/// After combining: 6 unique terms (no duplicates in 2-site case)
#[test]
#[ignore = "not yet implemented"]
fn hubbard_2site_term_count() {
    let input = r#"
        lattice 1d sites=2 pbc=false

        sum i=0..1:
          -t * c†(i,up) c(i+1,up) + h.c.
          -t * c†(i,down) c(i+1,down) + h.c.
          U * n(i,up) n(i,down)

        params:
          t = 1.0
          U = 4.0
    "#;

    let _ham = todo!("parse and run pipeline");
    // assert_eq!(ham.terms.len(), 6);
}

/// Test 2: 2-site Hubbard hopping coefficients are correct.
///
/// Each hopping term should have coefficient ±t = ±1.0
/// Interaction terms should have coefficient U = 4.0
#[test]
#[ignore = "not yet implemented"]
fn hubbard_2site_coefficients() {
    let input = r#"
        lattice 1d sites=2 pbc=false

        sum i=0..1:
          -t * c†(i,up) c(i+1,up) + h.c.
          -t * c†(i,down) c(i+1,down) + h.c.
          U * n(i,up) n(i,down)

        params:
          t = 1.0
          U = 4.0
    "#;

    let _ham = todo!("parse and run pipeline");
    // Hopping terms: coefficient should be -1.0
    // Hermitian conjugate terms: coefficient should be -1.0
    // Interaction terms: coefficient should be 4.0
}

/// Test 3: Periodic boundary conditions wrap correctly.
///
/// With PBC, site N connects back to site 0.
/// 4-site ring: should produce 8 hopping + 4 interaction = 12 terms.
#[test]
#[ignore = "not yet implemented"]
fn hubbard_4site_pbc() {
    let input = r#"
        lattice 1d sites=4 pbc=true

        sum i=0..4:
          -t * c†(i,up) c(i+1,up) + h.c.
          -t * c†(i,down) c(i+1,down) + h.c.
          U * n(i,up) n(i,down)

        params:
          t = 1.0
          U = 4.0
    "#;

    let _ham = todo!("parse and run pipeline");
    // assert_eq!(ham.terms.len(), 12);
}

/// Test 4: Normal ordering produces correct sign.
///
/// c(0,up) c†(0,up) should become -c†(0,up) c(0,up) + 1
/// (anticommutation: {c, c†} = 1)
#[test]
#[ignore = "not yet implemented"]
fn normal_ordering_fermion_sign() {
    let _result = todo!("create c c† term and normal order");
    // Should produce two terms:
    // -1.0 * c†(0,up) c(0,up)
    // +1.0 * (identity)
}

/// Test 5: Like terms with same operator string are combined.
///
/// Two terms with identical operators should have coefficients summed.
/// If sum is zero, the term should be removed.
#[test]
#[ignore = "not yet implemented"]
fn combine_like_terms() {
    let _result = todo!("create two terms with same ops, combine");
    // 2.0 * c†(0,up) c(1,up) + 3.0 * c†(0,up) c(1,up)
    // → 5.0 * c†(0,up) c(1,up)
}

/// Test 6: Zero-coefficient terms are eliminated after combining.
#[test]
#[ignore = "not yet implemented"]
fn combine_eliminates_zeros() {
    let _result = todo!("create cancelling terms, combine");
    // 1.0 * c†(0,up) c(1,up) + (-1.0) * c†(0,up) c(1,up)
    // → empty
}
