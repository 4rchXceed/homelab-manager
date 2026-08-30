use std::any::Any;

pub trait TypePart {
    fn parse(&mut self, data: &[u8]) -> Result<Vec<u8>, &str>;
    fn reset_value(&mut self);
    fn value(&self) -> Option<PartValue>;
    fn as_bytes(&self) -> Option<Vec<u8>>;
}

pub struct PartValue {
    value: Box<dyn Any + Send + Sync>,
}

impl PartValue {
    pub fn new<T: Any + Send + Sync>(value: T) -> Self {
        PartValue {
            value: Box::new(value),
        }
    }

    pub fn into<T: Any + Send + Sync>(self) -> Option<T> {
        return self.value.downcast::<T>().ok().map(|boxed| *boxed);
    }

    pub fn as_str(&self) -> Option<&str> {
        return self.value.downcast_ref::<String>().map(|s| s.as_str());
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        return self.value.downcast_ref::<Vec<u8>>().map(|v| v.as_slice());
    }
}
