#![forbid(unsafe_code)]

use json_schema_macro::*;
use serde_json::{json, Map, Value};

enum AddressType {
    US,
    UK,
    JP,
}

impl AddressType {
    pub fn from_country_code(s: &str) -> Result<Self, String> {
        match s {
            "US" => Ok(Self::US),
            "UK" => Ok(Self::UK),
            "JP" => Ok(Self::JP),
            unknown => Err(format!("unknown address type '{}'", unknown)),
        }
    }

    #[allow(dead_code)]
    pub fn to_country_code(&self) -> &'static str {
        match self {
            Self::US => "US",
            Self::UK => "UK",
            Self::JP => "JP",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::US => "US",
            Self::UK => "UK",
            Self::JP => "Japan",
        }
    }

    pub fn to_json(&self) -> Value {
        let description = format!("{} Address", self.name());

        let address_fields = match self {
            Self::US => vec!["street", "city", "state", "zip"],
            Self::UK => vec!["buildingName", "street", "city", "county", "postalCode"],
            Self::JP => vec!["postalCode", "prefecture", "city", "streetNumber"],
        };

        let props = address_fields.iter().fold(Map::new(), |mut acc, &current| {
            acc.insert(current.to_owned(), json!({ "type": "string" }));
            acc
        });

        json!({
            "description": description,
            "type": "object",
            "properties": props,
            "required": props.keys().collect::<Vec<_>>()
        })
    }
}

#[test]
fn test_addresses() {
    #[schema_macro]
    fn json_address_list(countries: Vec<String>) -> Result<Value, String> {
        Ok(countries
            .iter()
            .map(|country| {
                let address = AddressType::from_country_code(country).unwrap();
                address.to_json()
            })
            .collect())
    }

    let schema = eval_schema!(file = "json-schema-macro/tests/schemas/addresses.json").unwrap();
    let schema_str = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", schema_str);
}
