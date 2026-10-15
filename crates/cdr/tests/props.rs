//! Untrusted input never panics: arbitrary bytes as a package, a RIFF stream or a geometry record,
//! and arbitrary text as a package's XMP.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::Write;

use cdr::{Document, Limits};
use proptest::prelude::*;

fn package(metadata: &str, textinfo: &str) -> Vec<u8> {
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, data) in [("content/riffData.cdr", "RIFF\0\0\0\0CDRE"), ("metadata/metadata.xml", metadata), ("metadata/textinfo.xml", textinfo)] {
        w.start_file(name, opts).unwrap();
        w.write_all(data.as_bytes()).unwrap();
    }
    w.finish().unwrap().into_inner()
}

proptest! {
    #[test]
    fn arbitrary_geometry_records_never_panic(data in proptest::collection::vec(any::<u8>(), 0..512), v in 3u32..=26) {
        use cdr::geometry::{CorelFill, CorelPath, Outline, decode_loda, decode_mcfg, decode_trfd};
        let version = cdr::riff::CdrVersion(v * 100);
        let _ = decode_trfd(&data, version);
        let _ = decode_mcfg(&data, version);
        let _ = Outline::decode(&data, version).map(|o| o.to_stroke(vectorcraft_geom::Affine::IDENTITY, &mut Vec::new()));
        let _ = CorelFill::decode(&data, version).map(|(_, f)| f.to_fill(Some(vectorcraft_geom::Rect::new(0.0, 0.0, 10.0, 10.0)), &mut Vec::new()));
        if let Ok(l) = decode_loda(&data, version)
            && let Some(at) = l.coords
        {
            let _ = CorelPath::decode(&data, at, l.kind, version);
        }
    }

    #[test]
    fn arbitrary_riff_never_panics(body in proptest::collection::vec(any::<u8>(), 0..2048), version in b' '..=b'Z') {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&u32::try_from(body.len() + 4).unwrap().to_le_bytes());
        bytes.extend_from_slice(b"CDR");
        bytes.push(version);
        bytes.extend_from_slice(&body);
        let _ = cdr::riff::parse(&bytes, &Limits::default());
    }

    #[test]
    fn arbitrary_bytes_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let _ = Document::open(&bytes, Limits::default());
    }

    #[test]
    fn arbitrary_xmp_never_panics(
        meta in "(<[A-Za-z:]{1,6}( [a-z]{1,3}=\"[^\"]{0,4}\")?/?>|</[A-Za-z:]{1,6}>|[^<]{0,6}|&[a-z#0-9]{0,5};){0,40}",
        text in "(<rdf:Description>|<TextData>|<TextStream>|<TextRun break=\"line\">|</TextRun>|</TextStream>|[a-z ]{0,4}){0,30}",
    ) {
        let bytes = package(&meta, &text);
        let _ = Document::open(&bytes, Limits::default());
    }
}
