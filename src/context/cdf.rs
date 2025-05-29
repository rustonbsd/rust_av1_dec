#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Cdef {
    pub cdef_damping: u8,
}

impl Default for Cdef {
    fn default() -> Self {
        Self { cdef_damping: 3 }
    }
}
