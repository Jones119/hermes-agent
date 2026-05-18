
use serde_json::Value;

pub fn sanitize_surrogates(text: &str) -> String {
    text.chars()
        .map(|c| {
            let code = c as u32;
            if (0xD800..=0xDFFF).contains(&code) {
                '\u{FFFD}'
            } else {
                c
            }
        })
        .collect()
}

pub fn sanitize_structure_surrogates(payload: &mut Value) -> bool {
    let mut found = false;
    walk_and_sanitize(payload, &mut found);
    found
}

fn walk_and_sanitize(node: &mut Value, found: &mut bool) {
    match node {
        Value::String(s) => {
            let sanitized = sanitize_surrogates(s);
            if sanitized != *s {
                *s = sanitized;
                *found = true;
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                walk_and_sanitize(item, found);
            }
        }
        Value::Object(map) => {
            for value in map.values_mut() {
                walk_and_sanitize(value, found);
            }
        }
        _ => {}
    }
}

pub fn strip_non_ascii(text: &str) -> String {
    text.chars().filter(|c| c.is_ascii()).collect()
}

pub fn sanitize_messages_non_ascii(messages: &mut [Value]) -> bool {
    let mut found = false;
    for msg in messages.iter_mut() {
        if let Some(map) = msg.as_object_mut() {
            if let Some(Value::String(content)) = map.get_mut("content") {
                let sanitized = strip_non_ascii(content);
                if sanitized != *content {
                    *content = sanitized;
                    found = true;
                }
            }
            if let Some(Value::String(name)) = map.get_mut("name") {
                let sanitized = strip_non_ascii(name);
                if sanitized != *name {
                    *name = sanitized;
                    found = true;
                }
            }
            if let Some(Value::Array(tool_calls)) = map.get_mut("tool_calls") {
                for tc in tool_calls.iter_mut() {
                    if let Some(tc_map) = tc.as_object_mut() {
                        if let Some(Value::String(id)) = tc_map.get_mut("id") {
                            let s = strip_non_ascii(id);
                            if s != *id {
                                *id = s;
                                found = true;
                            }
                        }
                        if let Some(Value::Object(fn_obj)) = tc_map.get_mut("function") {
                            if let Some(Value::String(name)) = fn_obj.get_mut("name") {
                                let s = strip_non_ascii(name);
                                if s != *name {
                                    *name = s;
                                    found = true;
                                }
                            }
                            if let Some(Value::String(args)) = fn_obj.get_mut("arguments") {
                                let s = strip_non_ascii(args);
                                if s != *args {
                                    *args = s;
                                    found = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    found
}

pub fn repair_tool_call_arguments(raw_args: &str) -> String {
    let trimmed = raw_args.trim();
    if trimmed.is_empty() {
        return "{}".to_string();
    }

    if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
        return serde_json::to_string(&val).unwrap_or_else(|_| "{}".to_string());
    }

    let fixed = fix_common_json_errors(trimmed);
    if let Ok(val) = serde_json::from_str::<Value>(&fixed) {
        return serde_json::to_string(&val).unwrap_or_else(|_| "{}".to_string());
    }

    let escaped = escape_invalid_chars_in_json_strings(&fixed);
    if let Ok(val) = serde_json::from_str::<Value>(&escaped) {
        return serde_json::to_string(&val).unwrap_or_else(|_| "{}".to_string());
    }

    tracing::warn!("Failed to repair tool call arguments, returning empty JSON");
    "{}".to_string()
}

fn fix_common_json_errors(raw: &str) -> String {
    let mut result = raw.to_string();

    result = result.replace("None", "null");
    result = result.replace("True", "true");
    result = result.replace("False", "false");

    result = result.replace(",}", "}");
    result = result.replace(",]", "]");

    if !result.ends_with('}') && !result.ends_with(']') {
        let open_braces = result.chars().filter(|&c| c == '{').count();
        let close_braces = result.chars().filter(|&c| c == '}').count();
        for _ in 0..open_braces.saturating_sub(close_braces) {
            result.push('}');
        }
    }

    result
}

fn escape_invalid_chars_in_json_strings(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_string = false;
    let chars: Vec<char> = raw.chars().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        let ch = chars[i];
        if in_string {
            if ch == '\\' && i + 1 < n {
                out.push(ch);
                out.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if ch == '"' {
                in_string = false;
                out.push(ch);
            } else if (ch as u32) < 0x20 {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            } else {
                out.push(ch);
            }
        } else {
            if ch == '"' {
                in_string = true;
            }
            out.push(ch);
        }
        i += 1;
    }

    out
}

pub fn sanitize_tool_arguments(args: &str, tool_name: &str) -> String {
    let repaired = repair_tool_call_arguments(args);
    if repaired != args.trim() {
        tracing::warn!(
            "Tool '{}' arguments were repaired from malformed JSON",
            tool_name
        );
    }
    repaired
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_surrogates_no_change() {
        assert_eq!(sanitize_surrogates("hello"), "hello");
    }

    #[test]
    fn test_strip_non_ascii() {
        assert_eq!(strip_non_ascii("hello世界"), "hello");
    }

    #[test]
    fn test_repair_tool_call_arguments_empty() {
        assert_eq!(repair_tool_call_arguments(""), "{}");
    }

    #[test]
    fn test_repair_tool_call_arguments_trailing_comma() {
        let result = repair_tool_call_arguments(r#"{"key": "value",}"#);
        assert!(result.contains("key"));
    }

    #[test]
    fn test_repair_tool_call_arguments_python_none() {
        let result = repair_tool_call_arguments(r#"{"key": None}"#);
        assert!(result.contains("null"));
    }

    #[test]
    fn test_fix_common_json_errors() {
        assert_eq!(fix_common_json_errors(r#"{"a": True}"#), r#"{"a": true}"#);
        assert_eq!(fix_common_json_errors(r#"{"a": None}"#), r#"{"a": null}"#);
    }
}
