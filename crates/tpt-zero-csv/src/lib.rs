#![no_std]
//! `tpt-zero-csv`: a small CSV reader/writer.
//!
//! The `#![no_std]` core provides an [`Reader`] implementing the RFC 4180
//! model: comma-separated fields, optional double-quote quoting (with `""`
//! as an escaped quote), and CRLF or LF row terminators. Unquoted fields are
//! **borrowed** directly from the input ([`Field::Borrowed`]); quoted or
//! escaped fields are written into a caller-provided buffer
//! ([`Field::Buffered`]). The reader itself never allocates.
//!
//! Behind the `alloc` feature, [`read_records`] returns [`OwnedRecord`]s
//! (heap-allocated `Vec<String>` rows) and a [`Writer`] is provided for
//! serialization.
//!
//! # Scope (v0.1)
//!
//! Single-character delimiter only (`,` by default; configurable). No
//! streaming multi-MB quoting edge cases beyond RFC 4180; embedded newlines
//! inside quotes are supported.

/// A parsed CSV field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field<'a> {
    /// An unquoted field, borrowed directly from the input.
    Borrowed(&'a [u8]),
    /// A quoted/escaped field whose decoded contents live in the caller's
    /// scratch buffer. The slice is valid until the next [`Reader`] call.
    Buffered(&'a [u8]),
    /// A completely empty field (no bytes).
    Empty,
}

/// An error encountered while reading CSV.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CsvError {
    /// The provided scratch buffer was too small for a quoted/escaped field.
    BufferTooSmall,
    /// A quote was opened but never closed before end-of-input.
    UnterminatedQuote(usize),
}

/// A streaming RFC 4180 CSV reader. Borrows its input; never allocates.
///
/// Quoted/escaped fields are decoded into an internal scratch buffer. A
/// returned [`Field`] (via [`Reader::field`]) borrows the reader, so it stays
/// valid only until the next `next_row` call — copy field bytes out if you
/// need to retain them.
pub struct Reader<'a> {
    src: &'a [u8],
    pos: usize,
    delimiter: u8,
    /// Decoded bytes for the current row's quoted fields.
    scratch: [u8; 512],
    /// Per-field bounds for the current row: `(quoted, start, end)`.
    meta: [(bool, usize, usize); 256],
    row_len: usize,
}

impl<'a> Reader<'a> {
    /// Create a reader over `src` using `,` as the delimiter.
    #[inline]
    pub fn new(src: &'a [u8]) -> Self {
        Reader::with_delimiter(src, b',')
    }

    /// Create a reader with a custom single-byte delimiter.
    #[inline]
    pub fn with_delimiter(src: &'a [u8], delimiter: u8) -> Self {
        Reader {
            src,
            pos: 0,
            delimiter,
            scratch: [0u8; 512],
            meta: [(false, 0usize, 0usize); 256],
            row_len: 0,
        }
    }

    /// Current byte offset.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Number of fields in the most recently read row.
    #[inline]
    pub fn row_len(&self) -> usize {
        self.row_len
    }

    /// Read the next row. Returns `Ok(Some(n))` with `n` fields (query each
    /// via [`Reader::field`]), `Ok(None)` at end-of-input, or `Err(_)` on a
    /// malformed row (e.g. a scratch buffer too small for a quoted field).
    #[inline]
    pub fn next_row(&mut self) -> Result<Option<usize>, CsvError> {
        if self.pos >= self.src.len() {
            return Ok(None);
        }
        self.row_len = 0;
        let mut field_start = self.pos;
        let mut in_quotes = false;
        let mut quoted = false;
        let mut buf_len = 0usize;
        let mut q_start = 0usize;
        let mut copy_from = field_start;

        let commit = |meta: &mut [(bool, usize, usize); 256],
                      row_len: &mut usize,
                      quoted: bool,
                      start: usize,
                      end: usize| {
            if *row_len < meta.len() {
                meta[*row_len] = (quoted, start, end);
                *row_len += 1;
            }
        };

        loop {
            if self.pos >= self.src.len() {
                commit(
                    &mut self.meta,
                    &mut self.row_len,
                    quoted,
                    field_start,
                    self.pos,
                );
                return Ok(Some(self.row_len));
            }
            let b = self.src[self.pos];
            if in_quotes {
                if b == b'"' {
                    if self.pos + 1 < self.src.len() && self.src[self.pos + 1] == b'"' {
                        if buf_len + (self.pos - copy_from) > self.scratch.len() {
                            return Err(CsvError::BufferTooSmall);
                        }
                        self.scratch[buf_len..buf_len + (self.pos - copy_from)]
                            .copy_from_slice(&self.src[copy_from..self.pos]);
                        buf_len += self.pos - copy_from;
                        self.scratch[buf_len] = b'"';
                        buf_len += 1;
                        copy_from = self.pos + 2;
                        self.pos += 2;
                    } else {
                        if buf_len + (self.pos - copy_from) > self.scratch.len() {
                            return Err(CsvError::BufferTooSmall);
                        }
                        self.scratch[buf_len..buf_len + (self.pos - copy_from)]
                            .copy_from_slice(&self.src[copy_from..self.pos]);
                        buf_len += self.pos - copy_from;
                        copy_from = self.pos + 1;
                        self.pos += 1;
                        in_quotes = false;
                    }
                } else {
                    self.pos += 1;
                }
            } else if b == b'"' && self.pos == field_start {
                in_quotes = true;
                quoted = true;
                q_start = buf_len;
                copy_from = self.pos + 1;
                self.pos += 1;
            } else if b == self.delimiter {
                let start = if quoted { q_start } else { field_start };
                let end = if quoted { buf_len } else { self.pos };
                commit(&mut self.meta, &mut self.row_len, quoted, start, end);
                field_start = self.pos + 1;
                self.pos = field_start;
                in_quotes = false;
                quoted = false;
                // NOTE: `buf_len` is intentionally NOT reset here so that each
                // quoted field's decoded bytes occupy a distinct region of
                // `scratch` (preserving earlier fields' data).
                copy_from = field_start;
            } else if b == b'\n' || b == b'\r' {
                let start = if quoted { q_start } else { field_start };
                let end = if quoted { buf_len } else { self.pos };
                commit(&mut self.meta, &mut self.row_len, quoted, start, end);
                self.pos += 1;
                if b == b'\r' && self.pos < self.src.len() && self.src[self.pos] == b'\n' {
                    self.pos += 1;
                }
                return Ok(Some(self.row_len));
            } else {
                self.pos += 1;
            }
        }
    }

    /// The field at `index` in the current row. Borrows the reader; copy the
    /// bytes out if you need to retain them past the next `next_row`.
    #[inline]
    pub fn field(&self, index: usize) -> Field<'_> {
        let (quoted, start, end) = self.meta[index];
        if quoted {
            if start == end {
                Field::Buffered(&[])
            } else {
                Field::Buffered(&self.scratch[start..end])
            }
        } else if start == end {
            Field::Empty
        } else {
            Field::Borrowed(&self.src[start..end])
        }
    }
}

#[cfg(feature = "alloc")]
mod alloc_layer {
    extern crate alloc;
    use super::*;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    /// An owned CSV row (heap-allocated `String` fields).
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct OwnedRecord {
        pub fields: Vec<String>,
    }

    impl OwnedRecord {
        /// Number of fields.
        #[inline]
        pub fn len(&self) -> usize {
            self.fields.len()
        }

        /// Whether the record has no fields.
        #[inline]
        pub fn is_empty(&self) -> bool {
            self.fields.is_empty()
        }

        /// Field `i` as a string slice.
        #[inline]
        pub fn get(&self, i: usize) -> Option<&str> {
            self.fields.get(i).map(|s| s.as_str())
        }
    }

    /// Read all records from `src`, returning owned rows.
    pub fn read_records(src: &[u8]) -> Result<Vec<OwnedRecord>, CsvError> {
        let mut reader = Reader::new(src);
        let mut records = Vec::new();
        loop {
            match reader.next_row() {
                Ok(Some(n)) => {
                    let mut fields = Vec::with_capacity(n);
                    for i in 0..n {
                        let s = match reader.field(i) {
                            Field::Borrowed(b) | Field::Buffered(b) => {
                                core::str::from_utf8(b).unwrap_or("").to_string()
                            }
                            Field::Empty => String::new(),
                        };
                        fields.push(s);
                    }
                    records.push(OwnedRecord { fields });
                }
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(records)
    }

    /// A minimal CSV writer. Fields are quoted when they contain the
    /// delimiter, a quote, or a newline.
    pub struct Writer {
        out: String,
        delimiter: u8,
    }

    impl Writer {
        /// Create a writer with `,` delimiter.
        #[inline]
        pub fn new() -> Self {
            Writer {
                out: String::new(),
                delimiter: b',',
            }
        }

        /// Create a writer with a custom delimiter.
        #[inline]
        pub fn with_delimiter(delimiter: u8) -> Self {
            Writer {
                out: String::new(),
                delimiter,
            }
        }

        /// Append one record (a list of field strings).
        #[inline]
        pub fn write_record(&mut self, fields: &[&str]) {
            for (i, f) in fields.iter().enumerate() {
                if i > 0 {
                    self.out.push(self.delimiter as char);
                }
                let needs_quote = f.contains(self.delimiter as char)
                    || f.contains('"')
                    || f.contains('\n')
                    || f.contains('\r');
                if needs_quote {
                    self.out.push('"');
                    for c in f.chars() {
                        if c == '"' {
                            self.out.push('"');
                        }
                        self.out.push(c);
                    }
                    self.out.push('"');
                } else {
                    self.out.push_str(f);
                }
            }
            self.out.push('\n');
        }

        /// Finish and return the serialized CSV.
        #[inline]
        pub fn into_string(self) -> String {
            self.out
        }
    }

    impl Default for Writer {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(feature = "alloc")]
pub use alloc_layer::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_rows() {
        let input = b"a,b,c\n1,2,3\n";
        let mut r = Reader::new(input);
        let n1 = r.next_row().unwrap().unwrap();
        assert_eq!(n1, 3);
        assert_eq!(r.field(0), Field::Borrowed(b"a"));
        assert_eq!(r.field(2), Field::Borrowed(b"c"));
        let n2 = r.next_row().unwrap().unwrap();
        assert_eq!(r.field(1), Field::Borrowed(b"2"));
        assert_eq!(n2, 3);
        assert!(r.next_row().unwrap().is_none());
    }

    #[test]
    fn crlf_and_empty() {
        let input = b"x,y\r\n,,\r\n";
        let mut r = Reader::new(input);
        let n1 = r.next_row().unwrap().unwrap();
        assert_eq!(r.field(0), Field::Borrowed(b"x"));
        assert_eq!(n1, 2);
        let n2 = r.next_row().unwrap().unwrap();
        assert_eq!(n2, 3);
        assert_eq!(r.field(0), Field::Empty);
        assert_eq!(r.field(1), Field::Empty);
    }

    #[test]
    fn quoted_field() {
        let input = b"\"hello, world\",\"say \"\"hi\"\"\"\n";
        let mut r = Reader::new(input);
        let n = r.next_row().unwrap().unwrap();
        assert_eq!(n, 2);
        match r.field(0) {
            Field::Buffered(b) => assert_eq!(b, b"hello, world"),
            other => panic!("expected buffered: {other:?}"),
        }
        match r.field(1) {
            Field::Buffered(b) => assert_eq!(b, b"say \"hi\""),
            other => panic!("expected buffered: {other:?}"),
        }
    }

    #[test]
    fn quoted_embedded_newline() {
        let input = b"\"line1\nline2\",x\n";
        let mut r = Reader::new(input);
        let n = r.next_row().unwrap().unwrap();
        assert_eq!(n, 2);
        match r.field(0) {
            Field::Buffered(b) => assert_eq!(b, b"line1\nline2"),
            other => panic!("expected buffered: {other:?}"),
        }
    }

    #[test]
    fn single_row_no_trailing_newline() {
        let input = b"only,one,row";
        let mut r = Reader::new(input);
        let n = r.next_row().unwrap().unwrap();
        assert_eq!(n, 3);
        assert_eq!(r.field(2), Field::Borrowed(b"row"));
        assert!(r.next_row().unwrap().is_none());
    }

    #[cfg(feature = "alloc")]
    mod alloc_tests {
        extern crate alloc;
        use super::*;
        use alloc::vec;

        #[test]
        fn read_records_works() {
            let recs = read_records(b"a,b\n1,2\n3,4").unwrap();
            assert_eq!(recs.len(), 3);
            assert_eq!(recs[0].fields, vec!["a", "b"]);
            assert_eq!(recs[2].fields, vec!["3", "4"]);
        }

        #[test]
        fn roundtrip() {
            let mut w = Writer::new();
            w.write_record(&["name", "note"]);
            w.write_record(&["tpt", "hello, \"world\""]);
            let s = w.into_string();
            let recs = read_records(s.as_bytes()).unwrap();
            assert_eq!(recs[0].fields, vec!["name", "note"]);
            assert_eq!(recs[1].fields, vec!["tpt", "hello, \"world\""]);
        }

        #[test]
        fn quoted_escape_roundtrip() {
            let mut w = Writer::new();
            w.write_record(&["a\"b", "c\nd"]);
            let recs = read_records(w.into_string().as_bytes()).unwrap();
            assert_eq!(recs[0].fields, vec!["a\"b", "c\nd"]);
        }
    }
}

#[cfg(test)]
mod proptests {
    extern crate alloc;
    use super::*;
    use proptest::prelude::*;

    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    proptest! {
        /// The reader never panics on arbitrary bytes.
        #[test]
        fn never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
            let mut r = Reader::new(&bytes);
            for _ in 0..512 {
                match r.next_row() {
                    Ok(Some(_)) => {}
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
        }

        /// Round-tripping simple (unquoted, delimiter/quote-free) rows through
        /// the `alloc` writer and reader preserves the fields.
        #[test]
        #[cfg(feature = "alloc")]
        fn simple_roundtrip(rows in proptest::collection::vec(
            proptest::collection::vec("[a-z0-9]{1,8}", 1..5), 0..8)) {
            let mut w = Writer::new();
            for row in &rows {
                let refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
                w.write_record(&refs);
            }
            let out = w.into_string();
            let recs = read_records(out.as_bytes()).unwrap();
            prop_assert_eq!(recs.len(), rows.len());
            for (rec, orig) in recs.iter().zip(rows.iter()) {
                prop_assert_eq!(&rec.fields, orig);
            }
        }
    }
}
