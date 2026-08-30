use crate::builders::message_parts::type_part::{PartValue, TypePart};

#[derive(Clone)]
pub struct BinaryPart {
    size: usize,
    value: Option<Vec<u8>>,
}

impl BinaryPart {
    pub fn new(value: &[u8]) -> Self {
        return BinaryPart {
            size: 0,
            value: Some(value.to_vec()),
        };
    }
    pub fn size(size: usize) -> Self {
        return BinaryPart {
            size: size,
            value: None,
        };
    }
}

impl TypePart for BinaryPart {
    fn parse(&mut self, data: &[u8]) -> Result<Vec<u8>, &str> {
        if data.len() < self.size {
            return Err("Data is smaller than expected size");
        }
        let (start, end) = data.split_at(self.size);
        self.value = Some(start.to_vec());
        return Ok(end.to_vec());
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
            return Some(self.value.as_ref().unwrap().clone());
        } else {
            return None;
        }
    }
}
