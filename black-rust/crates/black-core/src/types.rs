//! C: include/types.h

/// C: `typedef s32 BOOL`
pub type Bool = i32;
/// C: `TRUE`
pub const TRUE: Bool = 1;
/// C: `FALSE`
pub const FALSE: Bool = 0;

/// Convert a Rust `bool` into a C `BOOL`.
pub fn to_bool(b: bool) -> Bool {
    if b {
        TRUE
    } else {
        FALSE
    }
}

/// Convert a C `BOOL` into a Rust `bool` (any non-zero value is true).
pub fn from_bool(b: Bool) -> bool {
    b != FALSE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_roundtrip() {
        assert_eq!(to_bool(true), 1);
        assert_eq!(to_bool(false), 0);
        assert!(from_bool(-5));
        assert!(!from_bool(0));
    }
}
