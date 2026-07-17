# tpt-zero-csv

A small CSV reader/writer for `#![no_std]`, with zero dependencies.

- `Reader` — RFC 4180 streaming parser. Unquoted fields are **borrowed**
  from the input ([`Field::Borrowed`]); quoted or escaped fields are decoded
  into a small internal buffer ([`Field::Buffered`]). The reader never
  allocates. CRLF and LF line endings, `""` escaped quotes, and embedded
  newlines inside quotes are all supported.
- `CsvError` — buffer-too-small / unterminated-quote error types.

Behind the `alloc` feature:

- [`read_records`] returns [`OwnedRecord`]s (`Vec<String>` rows).
- [`Writer`] serializes rows, quoting only when needed.

## Example

```rust
use tpt_zero_csv::{Field, Reader};

let input = b"a,b,c\n1,2,3\n";
let mut r = Reader::new(input);
let n = r.next_row().unwrap().unwrap();
assert_eq!(n, 3);
assert_eq!(r.field(0), Field::Borrowed(b"a"));
```

With the `alloc` feature:

```rust
use tpt_zero_csv::{read_records, Writer};

let mut w = Writer::new();
w.write_record(&["name", "note"]);
w.write_record(&["tpt", "hello, \"world\""]);
let recs = read_records(w.into_string().as_bytes()).unwrap();
assert_eq!(recs[1].fields, vec!["tpt", "hello, \"world\""]);
```

## Scope (v0.1)

Single-byte delimiter only (`,` by default, configurable). The reader stages
up to 256 fields per row and decodes quoted fields into a 512-byte internal
scratch buffer; larger quoted fields return `CsvError::BufferTooSmall`. A
returned [`Field`] borrows the reader and is only valid until the next
`next_row` call.

## `no_std`

`#![no_std]` core (reader) with zero external dependencies (beyond
`tpt-zero-utf8`, `tpt-zero-str-search`).

## License

Licensed under MIT or Apache-2.0 at your option.
