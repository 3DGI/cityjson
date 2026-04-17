/// Symbol interning policy for relational import/export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SymbolStorageMode {
    #[default]
    Adaptive,
    AlwaysIntern,
    NeverIntern,
}

/// String interning configuration for relational import/export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolStorageOptions {
    pub mode: SymbolStorageMode,
    pub small_input_string_threshold: usize,
    pub small_input_byte_threshold: usize,
}

impl Default for SymbolStorageOptions {
    fn default() -> Self {
        Self {
            mode: SymbolStorageMode::Adaptive,
            small_input_string_threshold: 256,
            small_input_byte_threshold: 16 * 1024,
        }
    }
}
