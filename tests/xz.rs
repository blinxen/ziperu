#![cfg(feature = "xz")]

use std::io::{Cursor, Read};
use ziperu::ZipArchive;

#[test]
fn decompress_xz() {
    let mut archive = ZipArchive::new(Cursor::new(include_bytes!("data/xz.zip")))
        .expect("zip file could not be read");
    let mut file = archive
        .by_name("test.txt")
        .expect("couldn't find file in archive");

    assert_eq!("test.txt", file.name());

    let mut content = Vec::new();
    file.read_to_end(&mut content)
        .expect("couldn't read encrypted and compressed file");
    assert_eq!(b"This is a test", &content[..]);
}
