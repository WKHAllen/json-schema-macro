use serde_json::Value;

fn find_macro_invocations_recursive(
    value: &Value,
    pointers: &mut Vec<String>,
    current_pointer: &str,
) {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        Value::Array(arr) => arr.iter().enumerate().for_each(|(index, arr_item)| {
            find_macro_invocations_recursive(
                arr_item,
                pointers,
                &format!("{}/{}", current_pointer, index),
            )
        }),
        Value::Object(obj) => obj.iter().for_each(|(obj_key, obj_value)| {
            find_macro_invocations_recursive(
                obj_value,
                pointers,
                &format!("{}/{}", current_pointer, obj_key),
            );

            if obj.len() == 1 && obj_key.starts_with("%{") && obj_key.ends_with("}%") {
                pointers.push(current_pointer.to_owned());
            }
        }),
    }
}

pub fn find_macro_invocations(value: &Value) -> Vec<String> {
    let mut pointers = Vec::new();
    find_macro_invocations_recursive(value, &mut pointers, "");
    pointers
}
