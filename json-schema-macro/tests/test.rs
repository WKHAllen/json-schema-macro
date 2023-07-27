#![forbid(unsafe_code)]

use json_schema_macro::*;
use serde_json::json;

#[test]
fn test_message() {
    #[schema_macro]
    fn fetch_message(_: ()) -> Result<&'static str, String> {
        Ok("Hello, world!")
    }

    let schema = eval_schema!(schema = {
        "message": {
            "%{fetch_message}%": null
        }
    })
    .unwrap();

    assert_eq!(schema, json!({ "message": "Hello, world!" }));
}
