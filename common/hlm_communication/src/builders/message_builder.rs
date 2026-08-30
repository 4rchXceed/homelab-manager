use std::collections::HashMap;

use crate::{
    builders::message_parts::{parsable_part::ParsableMessage, type_part::TypePart},
    errors::HLMCommunicationError,
};

pub type MessagePart = Box<dyn TypePart + Send + Sync>;
pub type TypeCallbackFn = Box<dyn Fn(ParsableMessage) -> Option<Vec<u8>> + Send + Sync>;

pub struct ProtocolMessage {
    type_part: Option<MessagePart>,
    callbacks: HashMap<Vec<u8>, TypeCallbackFn>,
    end_message: Option<Box<dyn TypePart + Sync + Send>>,
}

impl ProtocolMessage {
    pub fn new() -> Self {
        ProtocolMessage {
            type_part: None,
            callbacks: HashMap::new(),
            end_message: None,
        }
    }

    pub fn with_end_message<T>(mut self, part: T) -> Self
    where
        T: TypePart + Send + Sync + 'static,
    {
        self.end_message = Some(Box::new(part));
        self
    }

    pub fn end_message(&self) -> Option<Vec<u8>> {
        if self.end_message.is_some() {
            return self.end_message.as_ref().unwrap().as_bytes();
        } else {
            return None;
        }
    }

    pub fn add_type_part<T: TypePart + Send + Sync + 'static>(mut self, part: T) -> Self {
        self.type_part = Some(Box::new(part));
        self
    }

    pub fn for_type<F>(mut self, type_name: &[u8], callback: F) -> Self
    where
        F: Fn(ParsableMessage) -> Option<Vec<u8>> + Send + Sync + 'static,
    {
        self.callbacks
            .insert(type_name.to_vec(), Box::new(callback));
        self
    }

    pub(crate) fn parse_from(
        &mut self,
        mut data: Vec<u8>,
    ) -> Result<(Option<Vec<u8>>, bool), HLMCommunicationError> {
        if self.type_part.is_some() {
            let type_part = self.type_part.as_mut().unwrap();

            if self.end_message.is_some() {
                if self.end_message.as_ref().unwrap().as_bytes() == Some(data.clone()) {
                    return Ok((None, true));
                }
            }

            data = type_part
                .parse(&data)
                .map_err(|e| HLMCommunicationError::PartParsingError(format!("type: {}", e)))?;
            let type_value = type_part.as_bytes();
            if type_value.is_some() {
                let type_value = type_value.unwrap();

                for (required_type_val, callback) in self.callbacks.iter() {
                    if type_value == required_type_val.as_slice() {
                        let parsable_message = ParsableMessage::new(data.clone());
                        return Ok((callback(parsable_message), false));
                    }
                }
                return Ok((None, false));
            } else {
                return Err(HLMCommunicationError::PartParsingError(String::from(
                    "type part has no value after parsing",
                )));
            }
        } else {
            return Err(HLMCommunicationError::PartMissing(String::from("type")));
        }
    }
}
