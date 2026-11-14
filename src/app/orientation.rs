/// Choose the viewing orientation, in clockwise quarter turns, from an
/// accelerometer reading. Uncertain readings keep `previous`
pub fn choose(xyz: [i16; 3], previous: u32) -> u32 {
    let [x, y, _] = xyz;
    let [ax, ay, az] = xyz.map(|axis| axis.unsigned_abs() as u32);
    let (largest, other) = if ax > ay { (ax, ay) } else { (ay, ax) };
    let norm = ax * ax + ay * ay + az * az;
    // +/-2 g: 16384 counts/g. Ignore flat, diagonal, and moving readings
    // A clear edge must carry >=0.55 g and exceed other axes by >=0.15 g
    if !(12000 * 12000..=21000 * 21000).contains(&norm)
        || largest < 9000
        || largest < other + 2500
        || largest < az + 2500
    {
        return previous;
    }
    if ay > ax {
        return if y > 0 { 0 } else { 2 };
    }
    // Board clockwise gives +X; counter-rotate the glyphs
    if x > 0 { 3 } else { 1 }
}
