mod parse;

use std::ops::Range;

use hermit_core::UntypedForm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub value: T,

    // pub file: PathBuf,
    pub range: Range<usize>,
    pub start: LineColumn,
    pub end: LineColumn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ident(pub Spanned<String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent(pub Ident);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable(pub Ident);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form(pub UntypedForm<Ident, Ident>);

impl Default for Form {
    fn default() -> Self {
        Self(UntypedForm::Top)
    }
}
