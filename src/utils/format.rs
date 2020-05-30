use core::cmp::min;
use core::fmt;

pub struct Formatter<'a> {
    buffer: &'a mut [u8],
    index: usize,
}

impl<'a> Formatter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Formatter { buffer, index: 0 }
    }

    pub fn covert_to_str(self) -> Result<&'a str, fmt::Error> {
        if self.index > self.buffer.len() {
            return Err(fmt::Error);
        }
        use core::str::from_utf8;
        from_utf8(&self.buffer[..self.index]).map_err(|_e| { fmt::Error })
    }
}

impl<'a> fmt::Write for Formatter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.index > self.buffer.len() {
            return Err(fmt::Error);
        }
        let remaining_buffer = &mut self.buffer[self.index..];
        let raw_s = s.as_bytes();
        let write_num = min(raw_s.len(), remaining_buffer.len());
        remaining_buffer[..write_num].copy_from_slice(&raw_s[..write_num]);
        self.index += raw_s.len();
        if write_num < raw_s.len() {
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }
}

pub fn format<'a>(buffer: &'a mut [u8], args: fmt::Arguments) -> Result<&'a str, fmt::Error> {
    let mut formatter = Formatter::new(buffer);
    fmt::write(&mut formatter, args)?;
    formatter.covert_to_str()
}

#[macro_export]
macro_rules! format_buffer {
    ($write:expr,$($arg:tt)*) => {{
        let res = $crate::utils::format($write,format_args!($($arg)*));
        res
    }}
}
