use super::*;

#[test]
fn test_rope_char_to_line() {
    let s = "hello world\nhow are you".to_string();
    let buf = Rope::from_reader(s.as_bytes()).unwrap();
    assert_eq!(0, buf.char_to_line(0));
    assert_eq!('d', buf.char(10));
    assert_eq!(0, buf.char_to_line(10));
    assert_eq!('\n', buf.char(11));
    assert_eq!(0, buf.char_to_line(11));
    assert_eq!(1, buf.char_to_line(12));
}

#[test]
fn test_rope_line_len() {
    let s = "hello world\nhow are you".to_string();
    let buf = Rope::from_reader(s.as_bytes()).unwrap();
    let line = buf.line(buf.char_to_line(0));
    assert_eq!(12, line.len_chars());
}
