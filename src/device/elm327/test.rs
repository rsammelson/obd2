#![allow(clippy::panic)]

use std::{collections::VecDeque, io::Write as _};

use super::Elm327;
use crate::device::Obd2BaseDevice as _;

#[test]
fn basic() {
    let _device = Elm327::new(TestDevice::with_cmd_process(disallow_command)).unwrap();
}

#[test]
fn obd2() {
    let mut device = Elm327::new(TestDevice::with_cmd_process(|cmd, input| {
        assert_eq!(cmd, "0105");
        input.write_all(b"41 05 7B\r>").unwrap();
    }))
    .unwrap();
    assert_eq!(device.cmd(&[1, 5]).unwrap().unwrap(), "41 05 7B\n");
}

fn process(
    input: &mut VecDeque<u8>,
    output: &mut VecDeque<u8>,
    at_process: impl Fn(&str, &mut VecDeque<u8>) + Copy,
    cmd_process: impl Fn(&str, &mut VecDeque<u8>) + Copy,
) {
    // check line ending
    assert_eq!(output.pop_back(), Some(b'\n'));
    assert_eq!(output.pop_back(), Some(b'\r'));

    // decode to string
    let s = str::from_utf8(output.make_contiguous()).unwrap();

    // echo it back
    input.write_all(s.as_bytes()).unwrap();
    input.write_all(b"\r").unwrap();

    if let Some(cmd) = s.trim().to_ascii_uppercase().strip_prefix("AT") {
        // AT command
        at_process(cmd, input);
    } else if !s.trim().is_empty() {
        // other command
        cmd_process(s, input);
    }

    output.clear();
}

fn disallow_command(cmd: &str, _: &mut VecDeque<u8>) {
    panic!("got unexpected command: {cmd:?}");
}

pub struct TestDevice<T> {
    input: VecDeque<u8>,
    output: VecDeque<u8>,
    process: T,
}

impl TestDevice<Box<dyn Fn(&mut VecDeque<u8>, &mut VecDeque<u8>)>> {
    pub fn with_cmd_process(f: impl Fn(&str, &mut VecDeque<u8>) + Copy + 'static) -> Self {
        Self::with_process(Box::new(move |input, output| {
            process(input, output, disallow_command, f)
        }))
    }
}

impl<T> TestDevice<T> {
    fn with_process(process: T) -> Self {
        Self {
            input: VecDeque::new(),
            output: VecDeque::new(),
            process,
        }
    }
}

impl<T> std::io::Read for TestDevice<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.input.read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.input.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.input.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.input.read_exact(buf)
    }
}

impl<T> std::io::Write for TestDevice<T>
where
    T: FnMut(&mut VecDeque<u8>, &mut VecDeque<u8>),
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()?;
        (self.process)(&mut self.input, &mut self.output);
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.output.write_all(buf)
    }
}
