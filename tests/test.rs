#![forbid(unsafe_code)]

use json_schema_macro::*;
use serde_json::Value;

const US_ADDRESS: &str = r#"
{
  "description": "US Address",
  "type": "object",
  "properties": {
    "street": {"type": "string"},
    "city": {"type": "string"},
    "state": {"type": "string"},
    "zip": {"type": "string"}
  },
  "required": ["street", "city", "state", "zip"]
}
"#;

const UK_ADDRESS: &str = r#"
{
  "description": "UK Address",
  "type": "object",
  "properties": {
    "buildingName": {"type": "string"},
    "street": {"type": "string"},
    "city": {"type": "string"},
    "county": {"type": "string"},
    "postalCode": {"type": "string"}
  },
  "required": ["buildingName", "street", "city", "county", "postalCode"]
}
"#;

const JAPAN_ADDRESS: &str = r#"
{
  "description": "Japan Address",
  "type": "object",
  "properties": {
    "postalCode": {"type": "string"},
    "prefecture": {"type": "string"},
    "city": {"type": "string"},
    "streetNumber": {"type": "string"}
  },
  "required": ["postalCode", "prefecture", "city", "streetNumber"]
}
"#;

#[test]
fn test() {
    #[schema_macro]
    fn json_address_list(countries: Value) -> Result<Value, String> {
        match countries {
            Value::Array(countries) => countries
                .into_iter()
                .map(|country| match country {
                    Value::String(country) => match country.as_str() {
                        "US" => Ok(serde_json::from_str::<Value>(US_ADDRESS).unwrap()),
                        "UK" => Ok(serde_json::from_str::<Value>(UK_ADDRESS).unwrap()),
                        "JP" => Ok(serde_json::from_str::<Value>(JAPAN_ADDRESS).unwrap()),
                        other => Err(format!("unknown country code '{}'", other)),
                    },
                    _ => Err("expected countries to be an array of strings".to_owned()),
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array),
            _ => Err("expected countries to be an array of strings".to_owned()),
        }
    }

    let schema = eval_schema!(file = "tests/schemas/addresses.json").unwrap();
    dbg!(schema);
}