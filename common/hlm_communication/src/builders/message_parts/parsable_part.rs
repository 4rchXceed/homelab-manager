use indexmap::IndexMap;

use crate::builders::message_parts::type_part::{PartValue, TypePart};

pub struct ParsableMessage {
    raw_data: Vec<u8>,
}

impl ParsableMessage {
    pub fn new(raw_data: Vec<u8>) -> Self {
        ParsableMessage { raw_data }
    }

    pub fn get_raw_data(&self) -> &[u8] {
        &self.raw_data
    }

    pub fn parse<T: ProtocolMessageData>(self) -> Result<T, String> {
        let mut builded = T::build();
        let mut current_data = self.raw_data.clone();

        for (name, part) in builded.get_parts() {
            let remaining = part.parse(current_data.as_slice()); // Make part mutable
            if remaining.is_ok() {
                current_data = remaining.unwrap();
            } else {
                return Err(format!(
                    "Failed to parse part: {}, please check your code. Details: {}",
                    name,
                    remaining.unwrap_err()
                ));
            }
        }

        if !current_data.is_empty() {
            return Err(format!(
                "Not all data was consumed during parsing, please check your code. Bytes left: {} (Details: {:?})",
                current_data.len(),
                current_data
            ));
        }

        return T::when_parsed(&builded).map_or(
            Err(String::from("when_parsed failed, please check your code")),
            |parsed| Ok(parsed),
        );
    }
}

pub struct ParsedMessageParts {
    parts: IndexMap<String, Box<dyn TypePart + Send + Sync>>,
}

impl ParsedMessageParts {
    pub fn from(parts: IndexMap<String, Box<dyn TypePart + Send + Sync>>) -> Self {
        ParsedMessageParts { parts: parts }
    }

    pub fn new() -> Self {
        ParsedMessageParts {
            parts: IndexMap::new(),
        }
    }

    pub fn get_parts(&mut self) -> &mut IndexMap<String, Box<dyn TypePart + Send + Sync>> {
        return &mut self.parts;
    }

    pub fn add_part<T: TypePart + Send + Sync + 'static>(mut self, name: &str, part: T) -> Self {
        self.parts.insert(String::from(name), Box::new(part));
        self
    }

    pub fn get(&self, name: &str) -> Option<PartValue> {
        let part = self.parts.get(name);
        if part.is_some() {
            return part.as_ref().unwrap().value();
        } else {
            return None;
        }
    }
}

pub trait ProtocolMessageData
where
    Self: Sized,
{
    fn build() -> ParsedMessageParts;
    fn when_parsed(parts: &ParsedMessageParts) -> Option<Self>;
}
