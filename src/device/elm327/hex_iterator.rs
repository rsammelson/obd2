pub struct HexIterator {
    n: u8,
    state: NibblesDone,
}

enum NibblesDone {
    Zero,
    One,
    Two,
}

impl From<u8> for HexIterator {
    fn from(value: u8) -> Self {
        Self {
            n: value,
            state: NibblesDone::Zero,
        }
    }
}

impl Iterator for HexIterator {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let n = match self.state {
            NibblesDone::Zero => {
                self.state = NibblesDone::One;
                self.n >> 4
            }
            NibblesDone::One => {
                self.state = NibblesDone::Two;
                self.n & 0xf
            }
            NibblesDone::Two => return None,
        };
        Some(match n {
            0x0 => '0',
            0x1 => '1',
            0x2 => '2',
            0x3 => '3',
            0x4 => '4',
            0x5 => '5',
            0x6 => '6',
            0x7 => '7',
            0x8 => '8',
            0x9 => '9',
            0xA => 'A',
            0xB => 'B',
            0xC => 'C',
            0xD => 'D',
            0xE => 'E',
            0xF => 'F',
            _ => unreachable!(),
        })
    }
}

impl std::iter::FusedIterator for HexIterator {}

#[cfg(test)]
#[test]
fn test_basic() {
    assert_eq!(HexIterator::from(0).collect::<String>(), "00");
    assert_eq!(HexIterator::from(1).collect::<String>(), "01");
    assert_eq!(HexIterator::from(16).collect::<String>(), "10");
    assert_eq!(HexIterator::from(100).collect::<String>(), "64");
    assert_eq!(HexIterator::from(255).collect::<String>(), "FF");
}
