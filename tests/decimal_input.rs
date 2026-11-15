#[test]
fn duplicate_decimals_match_python_fraction_before_rounding() {
    #[derive(serde::Deserialize)]
    struct Case { terms:Vec<String>, expected:Option<u64> }
    let cases:Vec<Case>=serde_json::from_str(include_str!("fixtures/decimal-sum.json")).unwrap();
    for (i,case) in cases.into_iter().enumerate() {
        let mut text=String::from("NAME decimal\nROWS\n N c\nCOLUMNS\n");
        for term in case.terms {text+=&format!(" x c {term}\n");}text+="ENDATA\n";
        let actual=mip_codex::mps::parse(std::io::Cursor::new(text));
        match case.expected {
            Some(bits)=>assert_eq!(actual.unwrap().variables[0].objective.to_bits(),bits,"case{i}"),
            None=>assert!(actual.is_err(),"case{i}"),
        }
    }
}
