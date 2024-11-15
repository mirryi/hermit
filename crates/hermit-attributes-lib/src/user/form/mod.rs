mod parse;

use serde::{Deserialize, Serialize};
use syn::parse::{Parse, ParseStream, Result};

use parse::ParsedForm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form(pub epistemic::Form<Agent, Variable>);

impl Parse for Form {
    fn parse(input: ParseStream) -> Result<Self> {
        let form: ParsedForm = input.parse()?;
        Ok(form.into())
    }
}
