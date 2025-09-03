pub struct Elm327Reader<T> {
    device: T,
}

impl<T> Elm327Reader<T> {
    pub fn new(device: T) -> Self {
        Self { device }
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.device
    }
}

impl<T: std::io::Read> std::io::Read for Elm327Reader<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            match self.device.read(buf)? {
                0 => break Ok(0),
                n => {
                    let mut dst = 0;
                    for src in 0..n {
                        if matches!(buf[src], b'\0' | b'\n') {
                            // ignore this character
                        } else {
                            if let Ok([src, dst]) = buf.get_disjoint_mut([src, dst]) {
                                *dst = *src;
                            }
                            dst += 1;
                        }
                    }
                    if dst > 0 {
                        break Ok(dst);
                    } else {
                        // keep reading, we filtered out all characters
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Read as _;

    use super::Elm327Reader;

    #[test]
    fn no_invalid() {
        let mut reader = Elm327Reader::new(b"abcde".as_slice());
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        assert_eq!(buffer, b"abcde");
    }

    #[test]
    fn some_invalid() {
        let mut reader = Elm327Reader::new(b"a\0c\ne\r".as_slice());
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        assert_eq!(buffer, b"ace\r");
    }
}
