use hermit_core::Form;

#[derive(Debug, Clone)]
pub struct Location {}

#[derive(Debug, Clone)]
pub struct Agent {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Have {
    pub form: Form<Agent, Location>,
}

#[derive(Debug, Clone)]
pub struct Ensure {
    pub form: Form<Agent, Location>,
}

#[derive(Debug, Clone)]
pub struct Forgets {
    pub form: Form<Agent, Location>,
}

#[derive(Debug, Clone)]
pub struct FunctionName {}

#[derive(Debug, Clone)]
pub struct Call {
    pub name: FunctionName,
    pub args: Vec<Location>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub agents: Vec<Agent>,
    pub haves: Vec<Have>,
    pub ensures: Vec<Ensure>,
    pub forgets: Vec<Forgets>,

    pub calls: Vec<Call>,
}

#[derive(Debug, Clone)]
pub struct Info {
    pub fns: Vec<Function>,
}
