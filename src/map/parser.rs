//! Exports [`parse_map`].
#![allow(clippy::missing_docs_in_private_items)] // TODO: remove this
use std::{
    borrow::Cow,
    cmp::min,
    io::{self, Read},
};

use render::glm::vec3;
use thiserror::Error;
use world::{
    Vertex,
    brush::{BrushPlane, NGonPlane, TriPlane},
};

/// It is guaranteed that all `PlaneDatas`
/// have 3+ vertices.
pub struct PlaneData {
    verts: Box<[Vertex]>,
}
pub struct EntityData;

pub struct BrushData {
    pub planes: Box<[PlaneData]>,
}

pub struct MapData {
    entities: Box<[EntityData]>,
    brushes: Box<[BrushData]>,
}

impl MapData {
    pub fn new(entities: Box<[EntityData]>, brushes: Box<[BrushData]>) -> Self {
        Self { entities, brushes }
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("failed to read: {0}")]
    ReadError(#[from] io::Error),
    #[error("line 1: header \"{0}\" does not match magic header \"1113be_map\"")]
    BadHeader(String),
    #[error("line {0}: bad input ({1})")]
    BadInput(usize, String),
    #[error("line {0}: unexpected end of file: {1}")]
    UnexpectedEOF(usize, String),
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

// quickly exit if the file is malformed
// #[allow(clippy::useless_format)]
// macro_rules! malformed {
// ($line_count:expr, $str:expr) => {{
// let _line_count = $line_count;
// return Err(ParseError::BadInput(_line_count, format!("{}", $str)));
// }};
// }

/// Controls the line count value saved for outputting errors.
struct ParseErrorContext {
    line_count: usize,
}
impl ParseErrorContext {
    fn with_lines(line_count: usize) -> Self {
        Self { line_count }
    }
    fn bad_input(&self, err: impl Into<String>) -> ParseError {
        ParseError::BadInput(self.line_count, err.into())
    }
    fn unexpected_eof(&self, err: impl Into<String>) -> ParseError {
        ParseError::UnexpectedEOF(self.line_count, err.into())
    }
    fn io_err(&self, err: io::Error) -> ParseError {
        ParseError::ReadError(err)
    }
}

type LeftoverBytes = Box<[u8]>;
type Buffer = Box<[u8]>;

pub fn parse_map(mut input: impl Read) -> Result<MapData, ParseError> {
    check_header_valid(&mut input)?;
    // the header is correct, parse map as normal

    // 1: header
    // 2: start
    // LINE COUNT
    let mut err_ctx = ParseErrorContext::with_lines(2);

    let mut brushes: Option<Box<[BrushData]>> = None;
    let mut entities: Option<Box<[EntityData]>> = None;

    let mut last_loop_leftovers: Option<LeftoverBytes> = None;
    for _ in 0..2 {
        let leftover_bytes = last_loop_leftovers.take();
        // digits in 64 bits + section size
        let mut size_buf = [0; MAX_SECTION_HEADER_LEN];

        let mut buf_ptr = 0;
        let len = if let Some(leftover_bytes) = &leftover_bytes {
            let read_len = min(leftover_bytes.len(), size_buf.len());

            size_buf[..read_len].copy_from_slice(&leftover_bytes[..read_len]);

            read_len
                + input
                    .read(&mut size_buf[read_len..])
                    .map_err(|io| err_ctx.io_err(io))?
        } else {
            input.read(&mut size_buf).map_err(|io| err_ctx.io_err(io))?
        };

        let buf_safe = &size_buf[..len];

        if buf_safe.len() < MIN_SECTION_HEADER_LEN {
            return Err(err_ctx.unexpected_eof(format!(
                "expected length of at least {} for reading section header, found length {}",
                MIN_SECTION_HEADER_LEN,
                buf_safe.len()
            )));
        }

        let char = buf_safe[buf_ptr];
        if char != b'.' {
            err_ctx.bad_input(format!(
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
                    err_ctx.bad_input("entities section declared multiple times");
                }
                SectionType::Entities
            }
            b'B' => {
                if brushes.is_some() {
                    err_ctx.bad_input("brushes section declared multiple times");
                }
                SectionType::Brushes
            }
            other => {
                return Err(err_ctx.bad_input(format!(
                    "expected a valid section type, found '{}'",
                    other as char
                )));
            }
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
                return Err(err_ctx.bad_input(format!(
                    "expected digit in section header size part, found char '{}'",
                    *byte as char
                )));
            }
        }

        dbg!(buf_ptr);
        dbg!(ended_at);
        // eprintln!(
        //     "if we cut size_part to ended_at: '{}'",
        //     String::from_utf8_lossy(&size_part[..ended_at])
        // );
        // eprintln!(
        //     "buf_safe from here on: '{}'",
        //     String::from_utf8_lossy(&buf_safe[buf_ptr..])
        // );

        if size_part[ended_at] != b'\n' {
            err_ctx.bad_input(format!(
                "expected newline to end section header size part, found char '{}'",
                size_part[ended_at] as char
            ));
        }

        // after this point, buf_idx is after the newline

        let size_str =
            str::from_utf8(&size_part[..ended_at]).expect("all digits should be checked");

        let size_of_section: usize = size_str
            .parse()
            .expect("digits checked earlier should be able to make usize");

        dbg!(size_of_section);
        err_ctx.line_count += 1;

        let leftover_bytes = &buf_safe[buf_ptr..];

        let leftover_bytes = match section_type {
            SectionType::Brushes => {
                let (result, leftover) = parse_brushes_save_leftovers(
                    &mut input,
                    &mut err_ctx,
                    size_of_section,
                    leftover_bytes,
                )?;
                eprintln!("parsed brushes section successfully");
                brushes = Some(result);
                leftover
            }
            SectionType::Entities => {
                let (result, leftover) = parse_entities_save_leftovers(
                    &mut input,
                    &mut err_ctx,
                    size_of_section,
                    leftover_bytes,
                )?;
                eprintln!("parsed entities section successfully");
                entities = Some(result);
                leftover
            }
        };
        last_loop_leftovers = Some(leftover_bytes);
    }

    let (brushes, entities) = (brushes.unwrap(), entities.unwrap());
    let data = MapData::new(entities, brushes);
    Ok(data)
}

fn read_until_after_newline(
    buf: &[u8],
    err_ctx: &mut ParseErrorContext,
    ptr: &mut usize,
) -> Result<(), ParseError> {
    read_until_after_byte(buf, err_ctx, ptr, b'\n')
}

fn read_until_after_byte(
    buf: &[u8],
    err_ctx: &mut ParseErrorContext,
    ptr: &mut usize,
    byte: u8,
) -> Result<(), ParseError> {
    let mut our_ptr = *ptr;
    'reading: loop {
        let byte_at_idx = *buf.get(our_ptr).ok_or(err_ctx.unexpected_eof(format!(
            "EOF found while trying to read until '{:?}'",
            byte as char
        )))?;
        // dbg!(byte_at_idx as char);
        if byte != b'\n' && is_comment(buf, our_ptr) {
            eprintln!("byte is comment, reading until newline");
            read_until_after_newline(buf, err_ctx, &mut our_ptr)?;
            // ensure we don't accidentally skip another character
            // fixes breaking if newline goes directly into a comment
            continue 'reading;
        }
        if byte_at_idx == byte {
            eprintln!("matched byte {:?}", byte as char);
            break 'reading;
        } else if byte_at_idx == b'\n' {
            err_ctx.line_count += 1;
        }
        // avoid infinite recursion

        our_ptr += 1;
    }

    // skip over the byte
    our_ptr += 1;
    *ptr = our_ptr;
    Ok(())
}

fn parse_entities_save_leftovers(
    input: &mut impl Read,
    err_ctx: &mut ParseErrorContext,
    size: usize,
    leftover_bytes: &[u8],
) -> Result<(Box<[EntityData]>, LeftoverBytes), ParseError> {
    if size == 0 {
        return Ok((Box::new([]), leftover_bytes.into()));
    }
    let _buf = read_section_and_alloc_with_leftovers(input, err_ctx, size, leftover_bytes)?;
    todo!()
}

/// Returns data, new ptr index.
#[allow(unused_labels)]
fn parse_brush_data(
    buf: &[u8],
    err_ctx: &mut ParseErrorContext,
    has_positive_sign: bool,
) -> Result<(BrushData, usize), ParseError> {
    assert!(
        has_positive_sign,
        "can't deal with non positive signed input yet"
    );
    let mut ptr: usize = 0;
    let mut iter = buf.iter().cloned();

    macro_rules! next {
        () => {{
            // eprintln!("next!() called!");
            let next = iter.next();
            match next {
                Some(byte) => {
                    ptr += 1;
                    dbg!(byte as char);
                    byte
                },
                None => {
                    Err(err_ctx.bad_input("unexpected end of iterator while parsing vertices. note: 3 numbers must be specified per vertex".to_string()))?
                }
            }
        }};
    }

    let mut planes = vec![];
    'plane: loop {
        let vertices = vec![];
        'parsing_vertex: loop {
            let mut nums = vec![];
            if is_comment(buf, ptr) {
                // loop until newline
                eprintln!("'parsing_vertex: is_comment true, looping till newline");
                while next!() != b'\n' {}
                eprintln!("matched, stopping");
                err_ctx.line_count += 1;
            }
            'nums_loop: for _i in 0..3 {
                eprintln!("'nums_loop: iteration {_i}");
                let old_ptr = ptr;
                match next!() {
                    b'+' => (),
                    b'-' => (),
                    b'\n' => {
                        err_ctx.line_count += 1;
                        break 'parsing_vertex;
                    }
                    b => {
                        if is_comment(buf, ptr) {
                            // loop until newline
                            while next!() != b'\n' {}
                            err_ctx.line_count += 1;
                        }
                        return Err(err_ctx.bad_input(format!(
                        "expected positive or negative sign to begin vertex number set part, or newline to end set, got \"{}\"",
                        b as char
                    )));
                    }
                }
                'reading_num: loop {
                    let b = next!();
                    if b == b' ' {
                        break 'reading_num;
                    }
                    if !b.is_ascii_digit() && b != b'.' {
                        return Err(err_ctx.bad_input(
                            "expected parsable float values, found \"{b}\"".to_string(),
                        ));
                    }
                }
                let num_str = str::from_utf8(&buf[old_ptr..ptr - 1])
                    .expect("ascii digits should make valid utf8");
                let num: f32 = num_str.parse().map_err(|err| {
                    err_ctx.bad_input(format!(
                        "failed parsing number: {err:?} (num_str: {num_str}"
                    ))
                })?;
                dbg!(num);
                nums.push(num);
                // BELOW UNNECESSARY: we skip the space naturally doing next!() on the next loop
                // // skip over space
                // next!();
            }
            eprintln!("end of nums loop")
        }
        let plane = match vertices.len() {
            len @ ..3 => {
                return Err(err_ctx.bad_input(format!(
                    "vertices count too small! expected 3 or more, parsed {len}"
                )));
            }
            3 => BrushPlane::Triangle(TriPlane(vertices.try_into().unwrap())),
            4.. => BrushPlane::NGon(NGonPlane(vertices.into_boxed_slice())),
        };
        planes.push(plane);
    }
    // read_until_byte(buf, &mut ptr, b'e')?;

    Ok((
        BrushData {
            planes: Box::new([]),
        },
        ptr,
    ))
}

fn parse_brushes_save_leftovers(
    input: &mut impl Read,
    err_ctx: &mut ParseErrorContext,
    size: usize,
    leftover_bytes: &[u8],
) -> Result<(Box<[BrushData]>, LeftoverBytes), ParseError> {
    if size == 0 {
        return Ok((Box::new([]), leftover_bytes.into()));
    }
    // A box of all the bytes in the Brushes section.
    let (buf, ret_leftover) =
        read_section_and_alloc_with_leftovers(input, err_ctx, size, leftover_bytes)?;

    let mut ptr = 0;
    let mut brushes = vec![];
    'parse: loop {
        let byte = match buf.get(ptr) {
            Some(byte) => *byte,
            None => break 'parse,
        };
        dbg!(byte as char);
        dbg!(ptr);
        if byte == b'\n' {
            eprintln!("reached newline");
            // we're at a new line, skip to parsable byte
            ptr += 1;
            err_ctx.line_count += 1;
            continue 'parse;
        }
        if is_comment(&buf, ptr) {
            eprintln!("skipping comment");
            read_until_after_newline(&buf, err_ctx, &mut ptr)?;
            continue 'parse;
        }
        let pos_sign = match byte {
            b'p' => true,
            b'b' => false,
            b => {
                return Err(err_ctx.bad_input(format!(
                    "byte '{}' was not brush data starter 'b' or 'p'",
                    b as char
                )));
            }
        };
        // skip 'b', '\n'
        ptr += 2;
        err_ctx.line_count += 1;
        let (brush_data, offset) = parse_brush_data(&buf[ptr..], err_ctx, pos_sign)?;
        brushes.push(brush_data);
        ptr += offset;
    }
    Ok((brushes.into_boxed_slice(), ret_leftover))
}

fn is_comment(buf: &[u8], ptr: usize) -> bool {
    // if we're running in debug mode, print extra information to aid debugging
    if cfg!(debug_assertions) {
        let neighbourhood = &buf[ptr.saturating_sub(5)..ptr.saturating_add(6)];

        let mut sanitized_neighbors = vec![];
        for byte in neighbourhood.iter().cloned() {
            if byte == b'\n' {
                sanitized_neighbors.extend_from_slice(b"\\n");
            } else {
                sanitized_neighbors.push(byte);
            }
        }
        let neighborhood_string = String::from_utf8_lossy(&sanitized_neighbors);

        eprintln!("is_comment called: ptr: {ptr}");
        eprintln!("buf.get(ptr): '{:?}'", buf.get(ptr).map(|x| *x as char));
        eprintln!(
            "buf.get(ptr+1): '{:?}",
            buf.get(ptr + 1).map(|x| *x as char)
        );
        eprintln!("ptr neighborhood: \n'{neighborhood_string}'\n",);
    }

    dbg!(buf.get(ptr).is_some_and(|b| *b == b'/') && buf.get(ptr + 1).is_some_and(|b| *b == b'/'))
}

/// Read an input with dynamic size,
/// taking into account leftover bytes
fn read_section_and_alloc_with_leftovers(
    input: &mut impl Read,
    err_ctx: &mut ParseErrorContext,
    size: usize,
    leftover_bytes: &[u8],
) -> Result<(Buffer, LeftoverBytes), ParseError> {
    let ret_leftovers = if leftover_bytes.len() > size {
        leftover_bytes[size..].to_vec().into_boxed_slice()
    } else {
        Box::new([])
    };

    let mut buf = vec![0; size].into_boxed_slice();

    let min_len = min(leftover_bytes.len(), buf.len());

    buf[..min_len].copy_from_slice(&leftover_bytes[..min_len]);

    input.read_exact(&mut buf[min_len..]).map_err(|err| {
        err_ctx.bad_input(format!(
            "parser_read: could not read size {size} from input: {err}"
        ))
    })?;
    Ok((buf, ret_leftovers))
}

const FILE_HEADER: &[u8; 11] = b"1113be_map\n";
const FILE_HEADER_LEN: usize = FILE_HEADER.len();

fn check_header_valid(input: &mut impl Read) -> Result<(), ParseError> {
    let mut buf = [0; FILE_HEADER_LEN];
    input.read_exact(&mut buf)?;

    if &buf != b"1113be_map\n" {
        Err(ParseError::BadHeader(
            String::from_utf8_lossy(&buf).into_owned(),
        ))
    } else {
        Ok(())
    }
}
