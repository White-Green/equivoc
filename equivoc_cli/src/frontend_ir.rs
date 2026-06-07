#[derive(Debug, serde::Deserialize)]
pub struct EquivocFrontendIr {
    pub functions: Vec<EquivocFrontendFunction>,
    pub main_instructions: Vec<EquivocFrontendInstruction>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EquivocFrontendFunction {
    pub name: String,
    pub args: Vec<EquivocFrontendVariable>,
    pub instructions: Vec<EquivocFrontendInstruction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
pub struct EquivocFrontendVariable(pub u32);

impl From<EquivocFrontendVariable> for u32 {
    fn from(value: EquivocFrontendVariable) -> Self {
        value.0
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct FrontendIfVariableUpdate {
    pub variable: EquivocFrontendVariable,
    #[serde(rename = "then")]
    pub then_variable: EquivocFrontendVariable,
    #[serde(rename = "else")]
    pub else_variable: EquivocFrontendVariable,
}

#[derive(Debug, serde::Deserialize)]
pub struct FrontendLoopVariableUpdate {
    pub base: EquivocFrontendVariable,
    pub updated: EquivocFrontendVariable,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "tag")]
pub enum EquivocFrontendInstruction {
    If {
        variables: Vec<FrontendIfVariableUpdate>,
        condition: EquivocFrontendVariable,
        then_instructions: Vec<EquivocFrontendInstruction>,
        else_instructions: Vec<EquivocFrontendInstruction>,
    },
    For {
        variable_updates: Vec<FrontendLoopVariableUpdate>,
        loop_count: EquivocFrontendVariable,
        loop_index: EquivocFrontendVariable,
        instructions: Vec<EquivocFrontendInstruction>,
    },
    Loop {
        variable_updates: Vec<FrontendLoopVariableUpdate>,
        instructions: Vec<EquivocFrontendInstruction>,
    },
    Break,
    Continue,
    Return {
        value: Option<EquivocFrontendVariable>,
    },
    CallFunction {
        out: Option<EquivocFrontendVariable>,
        name: String,
        args: Vec<EquivocFrontendVariable>,
    },
    LoadIntegerConst {
        out: EquivocFrontendVariable,
        value: i64,
    },
    LoadFloatConst {
        out: EquivocFrontendVariable,
        value: f64,
    },
    LoadStringConst {
        out: EquivocFrontendVariable,
        value: String,
    },
    LoadBooleanConst {
        out: EquivocFrontendVariable,
        value: bool,
    },
    Add {
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
    },
    Sub {
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
    },
    Mul {
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
    },
    Div {
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
    },
    Mod {
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
    },
    Equals {
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
    },
    NotEquals {
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
    },
    LessThan {
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
    },
    LessThanOrEquals {
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
    },
    GreaterThan {
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
    },
    GreaterThanOrEquals {
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
    },
    LoadImage {
        out: EquivocFrontendVariable,
        path: EquivocFrontendVariable,
    },
    WriteImage {
        image: EquivocFrontendVariable,
        path: EquivocFrontendVariable,
    },
    ImageWidth {
        out: EquivocFrontendVariable,
        image: EquivocFrontendVariable,
    },
    ImageHeight {
        out: EquivocFrontendVariable,
        image: EquivocFrontendVariable,
    },
    ReadImagePixel {
        out: EquivocFrontendVariable,
        image: EquivocFrontendVariable,
        x: EquivocFrontendVariable,
        y: EquivocFrontendVariable,
    },
    WriteImagePixel {
        image: EquivocFrontendVariable,
        x: EquivocFrontendVariable,
        y: EquivocFrontendVariable,
        pixel: EquivocFrontendVariable,
    },
}
