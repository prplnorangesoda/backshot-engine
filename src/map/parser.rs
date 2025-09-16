use std::io::{self, BufRead, Read};

use paste::paste;

struct EntityData;
struct BrushData;

pub struct MapData {
    entities: Vec<EntityData>,
    brushes: Vec<BrushData>,
}

impl MapData {
    pub fn new() -> Self {
        Self {
            entities: vec![],
            brushes: vec![],
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    ReadError(io::Error),
    BadHeader(String),
    Malformed(String),
    UnexpectedEOF(String),
}

impl ParseError {
    fn bad_header(buf: &[u8]) -> Self {
        let string = String::from_utf8_lossy(buf);
        Self::BadHeader(format!(
            "[1:1]: header '{string}' does not match magic header '1113be_map'"
        ))
    }
}

impl From<io::Error> for ParseError {
    fn from(value: io::Error) -> Self {
        Self::ReadError(value)
    }
}

pub fn parse_map(mut input: impl Read) -> Result<MapData, ParseError> {
    let data = MapData::new();

    let mut buf = [0; 11];
    input.read_exact(&mut buf)?;

    if &buf != b"1113be_map\n" {
        Err(ParseError::bad_header(&buf))?;
    }
    // the header is correct, parse map as normal

    // digits in 64 bits + section size
    let mut size_buf = [0; 20 + 5];

    let len = input.read(&mut buf)?;

    let buf_safe = &buf[..len];

    // 1: header
    // 2: start
    let line_count = 2;
    let mut buf_idx = 0;

    macro_rules! malformed {
        ($str:expr) => {
            return Err(ParseError::Malformed(format!("{line_count}: {}", $str)))
        };
    }
    macro_rules! get {
        ($idx:expr) => {
            *buf_safe
                .get(buf_idx)
                .ok_or(ParseError::UnexpectedEOF(format!(
                    "{line_count}: unexpected EOF"
                )))?
        };
    }
    let char = buf_safe[buf_idx];
    if char != b'.' {
        malformed!(format!(
            "expected section starter '.', found '{}'",
            char as char
        ));
    }

    buf_idx += 1;
    let section_char = get!(buf_idx);
    enum SectionType {
        Brushes,
        Entities,
    }

    let section_type;
    if section_char == b'E' {
        section_type = SectionType::Entities
    } else if section_char == b'B' {
        section_type = SectionType::Brushes
    } else {
        malformed!(format!(
            "expected a valid section type, found '{}'",
            section_char as char
        ));
    }

    // match section_type {
    //     SectionType::Brushes => parse_brushes(&mut input, size, &leftover_bytes),
    //     SectionType::Entities => parse_entities(&mut input, size, &leftover_bytes),
    // }

    Ok(data)
}
