use std::{
    cmp::min,
    io::{self, Read},
};
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
            "1: header '{string}' does not match magic header '1113be_map'"
        ))
    }
}

impl From<io::Error> for ParseError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            io::ErrorKind::UnexpectedEof => Self::UnexpectedEOF(value.to_string()),
            _ => Self::ReadError(value),
        }
    }
}

const MIN_SECTION_HEADER_SIZE_PART_LEN: usize = "0\n".len();

#[cfg(target_pointer_width = "32")]
const MAX_SECTION_HEADER_SIZE_PART_LEN: usize = "4294967295\n".len();

#[cfg(target_pointer_width = "64")]
const MAX_SECTION_HEADER_SIZE_PART_LEN: usize = "18446744073709551615\n".len();

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("unrecognized pointer width, compile with 32 or 64-bit");

const MIN_SECTION_HEADER_TYPE_PART_LEN: usize = ".B ".len();
const MAX_SECTION_HEADER_TYPE_PART_LEN: usize = ".B ".len();

const MIN_SECTION_HEADER_LEN: usize =
    MIN_SECTION_HEADER_SIZE_PART_LEN + MIN_SECTION_HEADER_TYPE_PART_LEN;

const MAX_SECTION_HEADER_LEN: usize =
    MAX_SECTION_HEADER_SIZE_PART_LEN + MAX_SECTION_HEADER_TYPE_PART_LEN;

pub fn parse_map(mut input: impl Read) -> Result<MapData, ParseError> {
    let data = MapData::new();

    check_header_valid(&mut input)?;
    // the header is correct, parse map as normal

    // 1: header
    // 2: start
    let mut line_count = 2;

    let mut brushes: Option<BrushData> = None;
    let mut entities: Option<EntityData> = None;

    let mut last_loop_leftovers: Option<Box<[u8]>> = None;
    for _ in 0..2 {
        let leftover_bytes = last_loop_leftovers.take();
        // digits in 64 bits + section size
        let mut size_buf = [0; MAX_SECTION_HEADER_LEN];

        let mut buf_ptr = 0;
        let len = if let Some(leftover_bytes) = &leftover_bytes {
            let min_len = min(leftover_bytes.len(), size_buf.len());

            size_buf[..min_len].copy_from_slice(&leftover_bytes[..min_len]);

            min_len + input.read(&mut size_buf[min_len..])?
        } else {
            input.read(&mut size_buf)?
        };

        let buf_safe = &size_buf[..len];

        if buf_safe.len() < MIN_SECTION_HEADER_LEN {
            Err(ParseError::UnexpectedEOF(format!(
                "expected length of at least {} for reading section header, found length {}",
                MIN_SECTION_HEADER_LEN,
                buf_safe.len()
            )))?
        }
        // quickly exit if the file is malformed
        macro_rules! malformed {
            ($str:expr) => {
                return Err(ParseError::Malformed(format!("{line_count}: {}", $str)))
            };
        }

        let char = buf_safe[buf_ptr];
        if char != b'.' {
            malformed!(format!(
                "expected section starter '.', found '{}' (at buf_idx {buf_ptr})",
                char as char
            ));
        }

        enum SectionType {
            Brushes,
            Entities,
        }

        buf_ptr += 1;
        let section_char = buf_safe[buf_ptr];

        let section_type = match section_char {
            b'E' => {
                if entities.is_some() {
                    malformed!("entities section declared multiple times");
                }
                SectionType::Entities
            }
            b'B' => {
                if brushes.is_some() {
                    malformed!("brushes section declared multiple times");
                }
                SectionType::Brushes
            }
            other => malformed!(format!(
                "expected a valid section type, found '{}'",
                other as char
            )),
        };
        // skip over the space
        buf_ptr += 2;
        let size_part = &buf_safe[buf_ptr..];

        let mut ended_at = 0;
        for (i, byte) in size_part.iter().enumerate() {
            // eprintln!("BYTE: '{}'", String::from_utf8_lossy(&[*byte]));
            buf_ptr += 1;
            ended_at = i;
            if *byte == b'\n' {
                break;
            }
            if !byte.is_ascii_digit() {
                malformed!(format!(
                    "expected digit in section header size part, found char '{}'",
                    *byte as char
                ))
            }
        }

        dbg!(buf_ptr);
        dbg!(ended_at);
        dbg!(size_part.len());
        // eprintln!(
        //     "if we cut size_part to ended_at: '{}'",
        //     String::from_utf8_lossy(&size_part[..ended_at])
        // );
        // eprintln!(
        //     "buf_safe from here on: '{}'",
        //     String::from_utf8_lossy(&buf_safe[buf_ptr..])
        // );

        if size_part[ended_at] != b'\n' {
            malformed!(format!(
                "expected newline to end section header size part, found char '{}'",
                size_part[ended_at] as char
            ));
        }

        // after this point, buf_idx is after the newline

        let size_str =
            str::from_utf8(&size_part[..ended_at]).expect("all digits should be checked");

        let size: usize = size_str
            .parse()
            .expect("digits should be able to make usize");

        dbg!(size);
        line_count += 1;

        let leftover_bytes = &buf_safe[buf_ptr..];

        let leftover_bytes = match section_type {
            SectionType::Brushes => {
                let (result, leftover) = parse_brushes(&mut input, size, leftover_bytes)?;
                eprintln!("parsed brushes section successfully");
                brushes = Some(result);
                leftover
            }
            SectionType::Entities => {
                let (result, leftover) = parse_entities(&mut input, size, leftover_bytes)?;
                eprintln!("parsed entities section successfully");
                entities = Some(result);
                leftover
            }
        };
        // we have to copy and heap allocate here, otherwise we lose bytes
        last_loop_leftovers = Some(leftover_bytes.to_vec().into_boxed_slice());
    }

    Ok(data)
}

fn parse_entities<'a>(
    input: &mut impl Read,
    size: usize,
    leftover_bytes: &'a [u8],
) -> Result<(EntityData, &'a [u8]), ParseError> {
    if size == 0 {
        return Ok((EntityData, leftover_bytes));
    }
    let buf = parser_read_dynamic_alloc(input, size, leftover_bytes)?;
    todo!()
}

fn parse_brushes<'a>(
    input: &mut impl Read,
    size: usize,
    leftover_bytes: &'a [u8],
) -> Result<(BrushData, &'a [u8]), ParseError> {
    if size == 0 {
        return Ok((BrushData, leftover_bytes));
    }
    let buf = parser_read_dynamic_alloc(input, size, leftover_bytes)?;

    todo!()
}

/// Read an input with dynamic size,
/// taking into account leftover bytes
fn parser_read_dynamic_alloc(
    input: &mut impl Read,
    size: usize,
    leftover_bytes: &[u8],
) -> Result<Box<[u8]>, ParseError> {
    let mut buf = vec![0; size].into_boxed_slice();
    let min_len = min(leftover_bytes.len(), buf.len());

    buf[..min_len].copy_from_slice(&leftover_bytes[..min_len]);

    input.read_exact(&mut buf[min_len..]).map_err(|err| {
        ParseError::UnexpectedEOF(format!(
            "parser_read: could not read size {size} from input: {err}"
        ))
    })?;
    Ok(buf)
}

const FILE_HEADER: &[u8; 11] = b"1113be_map\n";
const FILE_HEADER_LEN: usize = FILE_HEADER.len();

fn check_header_valid(input: &mut impl Read) -> Result<(), ParseError> {
    let mut buf = [0; FILE_HEADER_LEN];
    input.read_exact(&mut buf)?;

    if &buf != b"1113be_map\n" {
        Err(ParseError::bad_header(&buf))
    } else {
        Ok(())
    }
}
