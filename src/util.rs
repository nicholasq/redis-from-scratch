use crate::resp::RespData;

pub fn assert_format_repr(value: &RespData, repr: &[u8]) {
    let mut buffer = Vec::new();
    value.write(&mut buffer).unwrap();
    assert_eq!(buffer, repr);
}
