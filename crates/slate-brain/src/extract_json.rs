use serde_json::Value;

/// Extract the first balanced JSON object or array from text.
///
/// Port of `src/main/brain.ts` `extractJson`: strip markdown fences, then
/// scan for the first `{` or `[` and match depth while ignoring string contents.
pub fn extract_json(text: &str) -> Result<Value, String> {
    // Strip ```json / ``` fences (same as /```(?:json)?/g in TS).
    let cleaned = text.replace("```json", "").replace("```", "");
    let cleaned = cleaned.trim();

    let starts: [(&str, &str); 2] = [("{", "}"), ("[", "]")];
    for (open, close) in starts {
        let Some(i) = cleaned.find(open) else {
            continue;
        };
        let open_ch = open.as_bytes()[0] as char;
        let close_ch = close.as_bytes()[0] as char;
        let bytes = cleaned.as_bytes();
        let mut depth: i32 = 0;
        let mut in_str = false;
        let mut esc = false;
        for j in i..cleaned.len() {
            let ch = bytes[j] as char;
            if esc {
                esc = false;
                continue;
            }
            if ch == '\\' {
                esc = true;
                continue;
            }
            if ch == '"' {
                in_str = !in_str;
            }
            if in_str {
                continue;
            }
            if ch == open_ch {
                depth += 1;
            } else if ch == close_ch {
                depth -= 1;
                if depth == 0 {
                    let slice = &cleaned[i..=j];
                    match serde_json::from_str::<Value>(slice) {
                        Ok(v) => return Ok(v),
                        Err(_) => break,
                    }
                }
            }
        }
    }
    Err("No valid JSON found in response".into())
}
