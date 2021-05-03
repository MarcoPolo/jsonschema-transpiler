use jst::convert_avro;
use jst::{Context, ResolveMethod};
use pretty_assertions::assert_eq;
use serde_json::Value;

#[test]
fn avro_test_array_with_atomics() {
    let input_data = r#"
    {
      "items": {
        "type": "integer"
      },
      "type": "array"
    }
    "#;
    let expected_data = r#"
    {
      "items": {
        "type": "long"
      },
      "type": "array"
    }
    "#;
    let mut context = Context {
        ..Default::default()
    };
    let input: Value = serde_json::from_str(input_data).unwrap();
    let expected: Value = serde_json::from_str(expected_data).unwrap();
    assert_eq!(expected, convert_avro(&input, context));

    context.resolve_method = ResolveMethod::Panic;
    convert_avro(&input, context);
}