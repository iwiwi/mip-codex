use mip_codex::mps::parse;
fn read(s: &str) -> mip_codex::model::Model {
    parse(std::io::Cursor::new(s)).unwrap()
}
#[test]
fn ranges_offset_and_duplicates() {
    let m = read(
        "NAME test\nOBJSENSE\n MAX\nROWS\n N cost\n L a\n G b\n E c\n E d\nCOLUMNS\n x cost 2 a 1\n x a 3 b 1\n y c 1 d 1\nRHS\n r cost 3 a 5\n r b 2 c 4\n r d 7\nRANGES\n r a -2 b 3\n r c 2 d -3\nBOUNDS\n FR b x\n FX b y 2\nENDATA\n",
    );
    assert!(m.maximize);
    assert_eq!(m.offset, -3.);
    assert_eq!(m.rows[0].coefficients, vec![(0, 4.)]);
    assert_eq!((m.rows[0].lower, m.rows[0].upper), (Some(3.), Some(5.)));
    assert_eq!((m.rows[1].lower, m.rows[1].upper), (Some(2.), Some(5.)));
    assert_eq!((m.rows[2].lower, m.rows[2].upper), (Some(4.), Some(6.)));
    assert_eq!((m.rows[3].lower, m.rows[3].upper), (Some(4.), Some(7.)));
    assert_eq!(m.variables[0].lower, None);
    assert_eq!(m.variables[1].upper, Some(2.));
}
#[test]
fn integer_defaults_and_bounds() {
    let m = read(
        "NAME test\nROWS\n N c\nCOLUMNS\n marker 'MARKER' 'INTORG'\n x c 1\n y c 1\n marker2 'MARKER' 'INTEND'\n z c 1\n w c 1\nBOUNDS\n LO b y 2\n UI b z 5\n UP b w -2\nENDATA\n",
    );
    assert!(m.variables[..3].iter().all(|v| v.integer));
    assert_eq!(m.variables[0].upper, Some(1.));
    assert_eq!(m.variables[1].upper, None);
    assert_eq!(m.variables[2].upper, Some(5.));
    assert_eq!(m.variables[3].lower, None);
}
#[test]
fn explicit_objective_and_exponents() {
    let m = read(
        "NAME t\nOBJNAME other\nROWS\n N first\n N other\nCOLUMNS\n x first 5 other 1D2\nENDATA\n",
    );
    assert_eq!(m.variables[0].objective, 100.);
    assert!(m.rows.is_empty());
}
#[test]
fn reject_unsafe_model_changes() {
    for s in [
        "NAME t\nROWS\n N c\nCOLUMNS\n x c NaN\nENDATA\n",
        "NAME t\nROWS\n N c\n N c\nENDATA\n",
        "NAME t\nROWS\n N c\nCOLUMNS\n x c 1\nBOUNDS\n SC b x 3\nENDATA\n",
        "NAME t\nROWS\n N c\nCOLUMNS\n x c 1\nRHS\n a c 1\n b c 2\nENDATA\n",
        "NAME t\nROWS\n N c\nCOLUMNS\n x c 1\n",
    ] {
        assert!(parse(std::io::Cursor::new(s)).is_err(), "{s}");
    }
}
