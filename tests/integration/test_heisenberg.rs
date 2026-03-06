/// Integration tests for Heisenberg model Hamiltonian processing.

/// Test 1: 2-site Heisenberg model expansion.
///
/// H = J (S+_0 S-_1 + S-_0 S+_1 + 2 Sz_0 Sz_1) (XXX model, isotropic)
///
/// Should produce 3 terms after expansion.
#[test]
#[ignore = "not yet implemented"]
fn heisenberg_2site_xxx() {
    let input = r#"
        lattice 1d sites=2 pbc=false

        sum i=0..1:
          J * Sp(i) Sm(i+1)
          J * Sm(i) Sp(i+1)
          J * 2.0 * Sz(i) Sz(i+1)

        params:
          J = 1.0
    "#;

    let _ham = todo!("parse and run pipeline");
    // assert_eq!(ham.terms.len(), 3);
}

/// Test 2: 4-site Heisenberg chain with PBC.
///
/// 4 sites × 3 terms per bond = 12 terms.
#[test]
#[ignore = "not yet implemented"]
fn heisenberg_4site_pbc() {
    let input = r#"
        lattice 1d sites=4 pbc=true

        sum i=0..4:
          J * Sp(i) Sm(i+1)
          J * Sm(i) Sp(i+1)
          J * 2.0 * Sz(i) Sz(i+1)

        params:
          J = 1.0
    "#;

    let _ham = todo!("parse and run pipeline");
    // assert_eq!(ham.terms.len(), 12);
}

/// Test 3: Spin commutation relation S+ S- = 2Sz + S- S+
///
/// Normal ordering should apply [S+, S-] = 2Sz correctly.
#[test]
#[ignore = "not yet implemented"]
fn spin_commutation_normal_order() {
    let _result = todo!("create S- S+ on same site, normal order");
    // S-(i) S+(i) → S+(i) S-(i) - 2 Sz(i)
    // (since [S+, S-] = 2Sz, so S- S+ = S+ S- - 2Sz)
}

/// Test 4: Sz symmetry reduction filters terms correctly.
///
/// In the Sz=0 sector, terms that change total Sz should be removed.
/// For Heisenberg: S+ S- and S- S+ preserve Sz, as does Sz Sz.
/// All terms of the isotropic Heisenberg model preserve Sz, so none are removed.
#[test]
#[ignore = "not yet implemented"]
fn sz_symmetry_preserves_heisenberg() {
    let input = r#"
        lattice 1d sites=4 pbc=true

        sum i=0..4:
          J * Sp(i) Sm(i+1)
          J * Sm(i) Sp(i+1)
          J * 2.0 * Sz(i) Sz(i+1)

        params:
          J = 1.0
    "#;

    let _ham = todo!("parse, pipeline, check no terms removed by Sz filter");
    // All 12 terms should survive Sz symmetry reduction
}
