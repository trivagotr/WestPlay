/// A bounded one-slot mailbox. Producers replace obsolete work.
#[derive(Debug)]
pub struct Latest<T>(Option<T>);
impl<T> Default for Latest<T> {
    fn default() -> Self {
        Self(None)
    }
}
impl<T> Latest<T> {
    pub fn put(&mut self, value: T) {
        self.0 = Some(value);
    }
    pub fn take(&mut self) -> Option<T> {
        self.0.take()
    }
}

pub struct Frame {
    pub bgra: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
