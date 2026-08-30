use hlm_communication::builders::message_parts::type_part::{PartValue, TypePart};
use serde_json::Value;

#[derive(Clone)]
pub struct JsonPart {
    value: Option<Value>,
}

impl JsonPart {
    pub fn new(value: Value) -> Self {
        return JsonPart { value: Some(value) };
    }

    pub fn create() -> Self {
        return JsonPart { value: None };
    }
}

impl TypePart for JsonPart {
    fn parse(&mut self, data: &[u8]) -> Result<Vec<u8>, &str> {
        let mut buf: Vec<u8> = Vec::new();
        let mut i = 0;
        let mut found = false;
        while i < data.len() && !found {
            let c = data[i] as char;
            if c == '\n' {
                found = true;
            } else {
                buf.push(data[i]);
                i += 1;
            }
        }
        if !found {
            return Err("Did not find the expected character");
        }
        let value_parsed = Some(String::from_utf8(buf).map_err(|_| "Invaid UTF8")?);
        let json_value: Value =
            serde_json::from_str(&value_parsed.unwrap()).map_err(|_| "Invalid JSON")?;
        self.value = Some(json_value);
        return Ok(data[i + 1..].to_vec());
    }
    fn reset_value(&mut self) {
        self.value = None;
    }
    fn value(&self) -> Option<PartValue> {
        if self.value.is_some() {
            return Some(PartValue::new(self.value.as_ref().unwrap().clone()));
        } else {
            return None;
        }
    }
    fn as_bytes(&self) -> Option<Vec<u8>> {
        if self.value.is_some() {
            let mut val = self.value.as_ref().unwrap().to_string();
            val.push('\n');
            return Some(val.as_bytes().to_vec());
        } else {
            return None;
        }
    }
}
