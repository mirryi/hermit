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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Ident(pub Spanned<String>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Agent(pub Ident);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Variable(pub Ident);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form(pub UntypedForm<Ident, Ident>);

impl Default for Form {
    fn default() -> Self {
        Self(UntypedForm::Top)
    }
}

impl<T> PartialEq for Spanned<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Spanned<T> where T: Eq {}

impl<T> PartialOrd for Spanned<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T> Ord for Spanned<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}
