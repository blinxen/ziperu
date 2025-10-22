#![cfg(feature = "deflate64")]

use std::io::{Cursor, Read};
use ziperu::ZipArchive;

#[test]
fn simple() {
    let mut archive = ZipArchive::new(Cursor::new(include_bytes!("data/deflate64/archive.zip")))
        .expect("zip file could not be read");
    let mut file = archive
        .by_name("binary.wmv")
        .expect("zip file does not contain binary.wmv");

    assert_eq!("binary.wmv", file.name());

    let mut content = Vec::new();
    file.read_to_end(&mut content)
        .expect("file in zip archive could not be read");
    assert_eq!(
        include_bytes!("data/deflate64/expected_bytes"),
        &content[..]
    );
}
