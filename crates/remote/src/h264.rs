/// Splits Annex-B on access-unit delimiters inserted by FFmpeg. A delimiter may
/// straddle any read boundary. Parameter sets stay with their IDR frame.
#[derive(Default)]
pub struct AccessUnits {
    buffer: Vec<u8>,
}
impl AccessUnits {
    pub fn push(&mut self, bytes: &[u8]) -> Vec<Vec<u8>> {
        self.buffer.extend_from_slice(bytes);
        let mut result = Vec::new();
        loop {
            let starts = delimiters(&self.buffer);
            if starts.len() < 2 {
                break;
            }
            let tail = self.buffer.split_off(starts[1]);
            result.push(std::mem::replace(&mut self.buffer, tail));
        }
        // Corrupt/unexpected output must not allocate without a limit.
        if self.buffer.len() > 8 * 1024 * 1024 {
            self.buffer.clear();
        }
        result
    }
    pub fn finish(&mut self) -> Vec<Vec<u8>> {
        if self.buffer.is_empty() { vec![] } else { vec![std::mem::take(&mut self.buffer)] }
    }
}
fn delimiters(data: &[u8]) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut i = 0;
    while i + 4 <= data.len() {
        let prefix = if data[i..].starts_with(&[0, 0, 0, 1]) {
            4
        } else if data[i..].starts_with(&[0, 0, 1]) {
            3
        } else {
            i += 1;
            continue;
        };
        if i + prefix < data.len() && data[i + prefix] & 31 == 9 {
            positions.push(i);
        }
        i += prefix;
    }
    positions
}
