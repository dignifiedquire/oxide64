use byteorder::{BigEndian, ReadBytesExt};
use errors::Result;

const HEADER_SIZE: usize = 0x1000;

// Big Endian (SUPER␣MARIO␣64␣␣)
const HEADER_NATIVE: [u8; 4] = [0x80, 0x37, 0x12, 0x40];
// Big Endian byte swapped (USEP␣RAMIR␣O46␣␣)
const HEADER_BYTE_SWAPPED: [u8; 4] = [0x37, 0x80, 0x40, 0x12];
// Little Endian (EPUSAM␣R␣OIR␣␣46)
const HEADER_LITTLE_ENDIAN: [u8; 4] = [0x40, 0x12, 0x37, 0x80];

#[derive(Debug, Clone, Copy)]
pub enum Endian {
    Little,
    Native,
    ByteSwapped,
}

impl Endian {
    fn from_u8(val: u8) -> Option<Endian> {
        if val == HEADER_NATIVE[0] {
            Some(Endian::Native)
        } else if val == HEADER_BYTE_SWAPPED[0] {
            Some(Endian::ByteSwapped)
        } else if val == HEADER_LITTLE_ENDIAN[0] {
            Some(Endian::Little)
        } else {
            None
        }
    }
}

/// Represents an in memory version of a parsed ROM.
#[derive(Debug)]
pub struct ROM {
    pub header: InternalHeader,
    pub data: Vec<u8>,
}

/// Represents an in memory version of a parse ROM header.
// TODO: fixed size array on the heap?
#[derive(Debug)]
pub struct InternalHeader {
    data: Vec<u8>,
}

impl InternalHeader {
    pub fn new(data: Vec<u8>) -> Result<InternalHeader> {
        if data.len() != HEADER_SIZE {
            return Err(format_err!(
                "invalid header size: {:#x} != {:#x}",
                data.len(),
                HEADER_SIZE
            ));
        } else {
            Ok(InternalHeader { data })
        }
    }

    pub fn pi_bsb_dom1_lat_reg(&self) -> u8 {
        self.data[0]
    }

    pub fn pi_bsd_dom1_pgs_reg(&self) -> u8 {
        self.data[1]
    }

    pub fn pi_bsd_dom1_pwd_reg(&self) -> u8 {
        self.data[2]
    }

    pub fn pi_bsb_dom1_pgs_reg(&self) -> u8 {
        self.data[3]
    }

    /// 0004h - 0007h     (1 dword): ClockRate
    pub fn clock_rate(&self) -> u64 {
        read_u64(&self.data[0x4..0x8])
    }

    /// 0008h - 000Bh     (1 dword): Program Counter (PC)
    pub fn pc(&self) -> u64 {
        read_u64(&self.data[0x8..0xC])
    }

    /// 000Ch - 000Fh     (1 dword): Release
    pub fn release(&self) -> u64 {
        read_u64(&self.data[0x8..0xC])
    }

    /// 0010h - 0013h     (1 dword): CRC1
    pub fn crc1(&self) -> u64 {
        read_u64(&self.data[0x10..0x14])
    }

    /// 0014h - 0017h     (1 dword): CRC2
    pub fn crc2(&self) -> u64 {
        read_u64(&self.data[0x14..0x18])
    }

    /// 0018h - 001Fh    (2 dwords): Unknown (0x0000000000000000)
    pub fn unknown_1(&self) -> [u64; 2] {
        [
            read_u64(&self.data[0x18..0x1C]),
            read_u64(&self.data[0x1C..0x20]),
        ]
    }

    /// 0020h - 0033h    (20 bytes): Image name
    ///                              Padded with 0x00 or spaces (0x20)
    pub fn image_name(&self) -> &[u8] {
        &self.data[0x20..0x33]
    }

    /// 0034h - 0037h     (1 dword): Unknown (0x00000000)
    pub fn unknown_2(&self) -> u64 {
        read_u64(&self.data[0x34..0x38])
    }

    /// 0038h - 003Bh     (1 dword): Manufacturer ID
    ///                              0x0000004E = Nintendo ('N')
    pub fn manufactorer_id(&self) -> u64 {
        // TODO: Enum
        read_u64(&self.data[0x38..0x3C])
    }

    /// 003Ch - 003Dh      (1 word): Cartridge ID
    pub fn cartridge_id(&self) -> u32 {
        read_u32(&self.data[0x3C..0x3E])
    }

    /// 003Eh - 003Fh      (1 word): Country code
    ///                              0x4400 = Germany ('D')
    ///                              0x4500 = USA ('E')
    ///                              0x4A00 = Japan ('J')
    ///                              0x5000 = Europe ('P')
    ///                              0x5500 = Australia ('U')
    pub fn country_code(&self) -> u32 {
        // TODO: enum
        read_u32(&self.data[0x3E..0x40])
    }

    /// 0040h - 0FFFh (1008 dwords): Boot code
    pub fn boot_code(&self) -> &[u8] {
        &self.data[0x40..0x1000]
    }
}

fn read_u32<T: ReadBytesExt>(mut data: T) -> u32 {
    data.read_u32::<BigEndian>().unwrap()
}

fn read_u64<T: ReadBytesExt>(mut data: T) -> u64 {
    data.read_u64::<BigEndian>().unwrap()
}

/// Parses a full ROM.
pub fn parse(data: Vec<u8>) -> Result<ROM> {
    let mut data = data;
    match Endian::from_u8(data[0]) {
        Some(e) => {
            match e {
                // Nothing to do, all good
                Endian::Native => {}
                Endian::ByteSwapped => {
                    // swap bytes
                    let mut i = 0;
                    while i < data.len() {
                        data.swap(i, i + 1);
                        i += 2;
                    }
                }
                Endian::Little => {
                    // convert to big endian
                    data.reverse();
                }
            }
        }
        None => return Err(format_err!("unknown header: {:#x}", data[0])),
    }

    let body = data.split_off(HEADER_SIZE);

    Ok(ROM {
        header: InternalHeader::new(data)?,
        data: body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use glob::glob;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_parse() {
        for entry in glob("./tests/N64-PD-ROMS/ROMS/*.z64").unwrap() {
            let entry = entry.unwrap();
            println!("reading {:?}", entry);
            let mut file = File::open(entry).unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();

            let rom = parse(bytes).expect("failed to parse");
            let header = rom.header;
            assert_eq!(header.pi_bsb_dom1_lat_reg(), HEADER_NATIVE[0]);
            assert_eq!(header.pi_bsd_dom1_pgs_reg(), HEADER_NATIVE[1]);
            assert_eq!(header.pi_bsd_dom1_pwd_reg(), HEADER_NATIVE[2]);
            // Some roms don't have the exact same bits here
            // assert_eq!(header.pi_bsb_dom1_pgs_reg(), HEADER_NATIVE[3]);
        }
    }
}
