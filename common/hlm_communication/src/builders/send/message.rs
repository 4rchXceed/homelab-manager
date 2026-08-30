use crate::builders::message_parts::type_part::TypePart;

pub struct MessageBuilder {
    pub message_type: Box<dyn TypePart + Send + Sync>,
    pub parts: Vec<Box<dyn TypePart + Send + Sync>>,
}

impl MessageBuilder {
    pub fn new<F>(message_type: F) -> Self
    where
        F: TypePart + Send + Sync + 'static,
    {
        return MessageBuilder {
            message_type: Box::new(message_type),
            parts: Vec::new(),
        };
    }

    pub fn add_part<F>(mut self, part: F) -> Self
    where
        F: TypePart + Send + Sync + 'static,
    {
        self.parts.push(Box::new(part));
        self
    }

    pub fn build(self) -> Option<Vec<u8>> {
        let mut message_bytes = self.message_type.as_bytes().unwrap_or_default();
        for part in self.parts {
            let part_bytes = part.as_bytes();
            if part_bytes.is_some() {
                message_bytes.extend_from_slice(part_bytes.unwrap().as_slice());
            } else {
                return None;
            }
        }
        return Some(message_bytes);
    }
}
