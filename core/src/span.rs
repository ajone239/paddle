use std::{
    cell::RefCell,
    fmt::{Debug, Display},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FileId(u16);

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
            return FileId(id as u16);
        }

        self.files.push(filename);
        let id = (self.files.len() - 1) as u16;
        FileId(id)
    }

    fn resolve(&self, id: FileId) -> &str {
        &self.files[id.0 as usize]
    }
}

thread_local! {
    static INTERNER: RefCell<Interner> = RefCell::new(Interner::new());
}

pub fn intern(filename: String) -> FileId {
    INTERNER.with_borrow_mut(|i| i.intern(filename))
}

#[derive(Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub line: u16,
    pub column: u16,
    pub file_name_id: FileId,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let file = INTERNER.with_borrow(|i| i.resolve(self.file_name_id).to_string());
        write!(f, "{}:{}:{}", file, self.line, self.column)
    }
}

// TODO(austin.jones): this needs to be killed
impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = f;
        Ok(())
    }
}
