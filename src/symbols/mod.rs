use std::fmt;
use std::ops::Range;
use std::path::PathBuf;

pub mod merkle;

pub type SymbolId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Module,
    Struct,
    Enum,
    Trait,
    Impl,
    Function,
    Method,
    Constant,
    TypeAlias,
    Static,
    Macro,
    Field,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolKind::Module => write!(f, "mod"),
            SymbolKind::Struct => write!(f, "struct"),
            SymbolKind::Enum => write!(f, "enum"),
            SymbolKind::Trait => write!(f, "trait"),
            SymbolKind::Impl => write!(f, "impl"),
            SymbolKind::Function => write!(f, "fn"),
            SymbolKind::Method => write!(f, "fn"),
            SymbolKind::Constant => write!(f, "const"),
            SymbolKind::TypeAlias => write!(f, "type"),
            SymbolKind::Static => write!(f, "static"),
            SymbolKind::Macro => write!(f, "macro"),
            SymbolKind::Field => write!(f, "field"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolNode {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: PathBuf,
    pub byte_range: Range<usize>,
    pub line_range: Range<usize>,
    pub content_hash: [u8; 32],
    pub merkle_hash: [u8; 32],
    pub children: Vec<SymbolNode>,
    pub estimated_tokens: usize,
}

impl SymbolNode {
    pub fn total_symbols(&self) -> usize {
        1 + self.children.iter().map(|c| c.total_symbols()).sum::<usize>()
    }

    pub fn total_tokens(&self) -> usize {
        self.estimated_tokens + self.children.iter().map(|c| c.total_tokens()).sum::<usize>()
    }
}

/// A file's worth of symbols, organized hierarchically.
#[derive(Debug, Clone)]
pub struct FileSymbols {
    pub file_path: PathBuf,
    pub symbols: Vec<SymbolNode>,
    pub total_lines: usize,
}

impl FileSymbols {
    pub fn total_symbols(&self) -> usize {
        self.symbols.iter().map(|s| s.total_symbols()).sum()
    }
}

/// The full project symbol tree, organized by directory structure.
#[derive(Debug, Clone)]
pub struct ProjectTree {
    pub root: PathBuf,
    pub files: Vec<FileSymbols>,
}

impl ProjectTree {
    pub fn total_symbols(&self) -> usize {
        self.files.iter().map(|f| f.total_symbols()).sum()
    }

    pub fn total_files(&self) -> usize {
        self.files.len()
    }
}
