use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FileId(usize);

pub struct Interner {
    files: Vec<String>,
}

impl Interner {
    fn new() -> Self {
        Self {
            files: vec!["DEFAULT".to_string()],
        }
    }
    fn intern(&mut self, filename: String) -> FileId {
        if let Some(id) = self.files.iter().position(|s| s == &filename) {
            return FileId(id);
        }

        self.files.push(filename);
        FileId(self.files.len() - 1)
    }

    fn resolve(&self, id: FileId) -> &str {
        &self.files[id.0]
    }
}

thread_local! {
    static INTERNER: RefCell<Interner> = RefCell::new(Interner::new());
}

pub fn intern(filename: String) -> FileId {
    INTERNER.with_borrow_mut(|i| i.intern(filename))
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub file_name_id: FileId,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let file = INTERNER.with_borrow(|i| i.resolve(self.file_name_id).to_string());
        write!(f, "{}:{}:{}", file, self.line, self.column)
    }
}
