#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryMode {
    Stack,
    Safe,
    Own,
}

impl MemoryMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "stack" | "hizli" | "fast" | "0" => Some(MemoryMode::Stack),
            "safe" | "guvenli" | "1" => Some(MemoryMode::Safe),
            "own" | "ownership" | "owner" | "2" => Some(MemoryMode::Own),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MemoryMode::Stack => "stack",
            MemoryMode::Safe => "safe",
            MemoryMode::Own => "own",
        }
    }

    pub fn at_least_safe(self) -> bool {
        self >= MemoryMode::Safe
    }
}
