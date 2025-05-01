#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#[derive(Eq, PartialEq)]
pub enum SeekType {
    RVNG_SEEK_CUR,
    RVNG_SEEK_SET,
    RVNG_SEEK_END,
}
