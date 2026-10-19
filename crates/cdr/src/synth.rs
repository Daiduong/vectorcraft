//! Original, tiny RIFF fixtures generated from the public record grammar.
//! Available only for tests or when the `synth` feature is enabled.

/// One 100 × 80 point CDR 12 page with a 40 × 20 point rectangle,
/// solid red RGB fill and a one-point black outline. Geometry starts at the
/// page centre (50, 40 points), extending right and down.
pub fn rectangle() -> Vec<u8> {
    fn u32s(values: &[u32]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }
    fn coord(points: f64) -> u32 {
        (points / 72.0 * 254000.0).round() as i32 as u32
    }
    fn chunk(id: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut v = id.to_vec();
        v.extend_from_slice(&(body.len() as u32).to_le_bytes());
        v.extend_from_slice(body);
        if !body.len().is_multiple_of(2) {
            v.push(0);
        }
        v
    }
    fn list(kind: &[u8; 4], children: &[u8]) -> Vec<u8> {
        let mut b = kind.to_vec();
        b.extend_from_slice(children);
        chunk(b"LIST", &b)
    }
    let rgb = |r: u8, g: u8, b: u8| {
        let mut v = vec![5, 0, 0, 0, 0, 0, 0, 0];
        v.extend_from_slice(&[b, g, r, 0]);
        v
    };
    let mut fill = vec![1, 0, 0, 0, 1, 0, 0, 0];
    fill.extend(rgb(255, 0, 0));
    let mut outline = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    outline.extend(u32s(&[coord(1.0)]));
    outline.extend_from_slice(&[100, 0, 0, 0]);
    outline.extend_from_slice(&[0; 56]);
    outline.extend(rgb(0, 0, 0));
    outline.extend_from_slice(&[0; 48]);
    let shape = u32s(&[coord(40.0), coord(-20.0), 0, 0, 0, 0]);
    let mut loda = u32s(&[84, 3, 60, 72, 1]);
    loda.extend_from_slice(&1u32.to_le_bytes());
    loda.extend_from_slice(&2u32.to_le_bytes());
    loda.extend_from_slice(&shape);
    loda.extend_from_slice(&[0; 8]);
    loda.extend(u32s(&[20, 24, 28, 30, 10, 20]));
    let object = list(b"obj ", &chunk(b"loda", &loda));
    let mut page = chunk(b"flgs", &[0, 0, 0, 0x90]);
    page.extend(object);
    let mut body = chunk(b"vrsn", &1200u16.to_le_bytes());
    let mut config = vec![0; 4];
    config.extend(u32s(&[coord(100.0), coord(80.0)]));
    body.extend(chunk(b"mcfg", &config));
    body.extend(chunk(b"fild", &fill));
    body.extend(chunk(b"outl", &outline));
    body.extend(list(b"page", &page));
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&((body.len() + 4) as u32).to_le_bytes());
    out.extend_from_slice(b"CDRC");
    out.extend(body);
    out
}
