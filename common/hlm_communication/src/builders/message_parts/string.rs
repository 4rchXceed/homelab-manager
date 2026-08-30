use crate::builders::message_parts::type_part::{PartValue, TypePart};

#[derive(Clone)]
pub struct StringPart {
    read_until: char,
    value: Option<String>,
}

impl StringPart {
    pub fn new(value: &str, end: char) -> Self {
        let mut val = String::from(value);
        val.push(end);
        return StringPart {
            read_until: '\n', // Default value, we don't need it when it's used as a value part
            value: Some(val),
        };
    }

    pub fn until(read_until: char) -> Self {
        return StringPart {
            read_until: read_until,
            value: None,
        };
    }
}

impl TypePart for StringPart {
    fn parse(&mut self, data: &[u8]) -> Result<Vec<u8>, &str> {
        let mut buf: Vec<u8> = Vec::new();
        let mut i = 0;
        let mut found = false;
        while i < data.len() && !found {
            let c = data[i] as char;
            if c == self.read_until {
                found = true;
            } else {
                buf.push(data[i]);
                i += 1;
            }
        }
        if !found {
            return Err("Did not find the expected character");
        }
        self.value = Some(String::from_utf8(buf).map_err(|_| "Invaid UTF8")?);
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
            return Some(self.value.as_ref().unwrap().as_bytes().to_vec());
        } else {
            return None;
        }
    }
}
