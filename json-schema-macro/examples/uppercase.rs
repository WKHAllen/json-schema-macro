use json_schema_macro::*;
use serde_json::json;

#[schema_macro]
fn uppercase(s: String) -> Result<String, String> {
    Ok(s.to_uppercase())
}

fn main() {
    let schema = eval_schema!(schema = {
        "message": {
            "%{uppercase}%": "Uppercase string"
        }
    })
    .unwrap();

    assert_eq!(schema, json!({ "message": "UPPERCASE STRING" }));
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
