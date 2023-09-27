use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub language); // synthesized by LALRPOP

#[test]
fn calculator1() {
    assert!(language::QuoteStringParser::new().parse(r#""hello vous allez bien ?""#).is_ok());
}