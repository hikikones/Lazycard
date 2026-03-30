/// Formats an integer to its string representation.
pub fn format_int(i: impl itoa::Integer, mut f: impl FnMut(&str)) {
    let mut buffer = itoa::Buffer::new();
    f(buffer.format(i))
}

/// Formats two integers to its string representations.
pub fn format_int2(i1: impl itoa::Integer, i2: impl itoa::Integer, mut f: impl FnMut(&str, &str)) {
    let (mut b1, mut b2) = (itoa::Buffer::new(), itoa::Buffer::new());
    f(b1.format(i1), b2.format(i2))
}
