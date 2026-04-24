#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
pub struct EquivocMirVariable(u32);

impl From<u32> for EquivocMirVariable {
    fn from(value: u32) -> Self {
        EquivocMirVariable(value)
    }
}

impl From<EquivocMirVariable> for u32 {
    fn from(value: EquivocMirVariable) -> Self {
        value.0
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct EquivocMir {
    pub functions: Vec<EquivocMirFunction>,
    pub main_instructions: Vec<EquivocMirInstruction>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EquivocMirFunction {
    pub name: String,
    pub args: Vec<EquivocMirVariable>,
    pub instructions: Vec<EquivocMirInstruction>,
}

#[derive(Debug, serde::Deserialize)]
pub struct IfVariableUpdate {
    pub variable: EquivocMirVariable,
    #[serde(rename = "then")]
    pub then_variable: EquivocMirVariable,
    #[serde(rename = "else")]
    pub else_variable: EquivocMirVariable,
}

#[derive(Debug, serde::Deserialize)]
pub struct LoopVariableUpdate {
    pub base: EquivocMirVariable,
    pub updated: EquivocMirVariable,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "tag")]
pub enum EquivocMirInstruction {
    If {
        variables: Vec<IfVariableUpdate>,
        condition: EquivocMirVariable,
        then_instructions: Vec<EquivocMirInstruction>,
        else_instructions: Vec<EquivocMirInstruction>,
    },
    For {
        variable_updates: Vec<LoopVariableUpdate>,
        loop_count: EquivocMirVariable,
        loop_index: EquivocMirVariable,
        instructions: Vec<EquivocMirInstruction>,
    },
    Loop {
        variable_updates: Vec<LoopVariableUpdate>,
        instructions: Vec<EquivocMirInstruction>,
    },
    Break,
    Continue,
    Return {
        value: Option<EquivocMirVariable>,
    },
    CallFunction {
        out: Option<EquivocMirVariable>,
        name: String,
        args: Vec<EquivocMirVariable>,
    },
    LoadIntegerConst {
        out: EquivocMirVariable,
        value: i64,
    },
    LoadFloatConst {
        out: EquivocMirVariable,
        value: f64,
    },
    LoadStringConst {
        out: EquivocMirVariable,
        value: String,
    },
    LoadBooleanConst {
        out: EquivocMirVariable,
        value: bool,
    },
    Add {
        out: EquivocMirVariable,
        lhs: EquivocMirVariable,
        rhs: EquivocMirVariable,
    },
    Sub {
        out: EquivocMirVariable,
        lhs: EquivocMirVariable,
        rhs: EquivocMirVariable,
    },
    Mul {
        out: EquivocMirVariable,
        lhs: EquivocMirVariable,
        rhs: EquivocMirVariable,
    },
    Div {
        out: EquivocMirVariable,
        lhs: EquivocMirVariable,
        rhs: EquivocMirVariable,
    },
    Mod {
        out: EquivocMirVariable,
        lhs: EquivocMirVariable,
        rhs: EquivocMirVariable,
    },
    Equals {
        out: EquivocMirVariable,
        lhs: EquivocMirVariable,
        rhs: EquivocMirVariable,
    },
    NotEquals {
        out: EquivocMirVariable,
        lhs: EquivocMirVariable,
        rhs: EquivocMirVariable,
    },
    LessThan {
        out: EquivocMirVariable,
        lhs: EquivocMirVariable,
        rhs: EquivocMirVariable,
    },
    LessThanOrEquals {
        out: EquivocMirVariable,
        lhs: EquivocMirVariable,
        rhs: EquivocMirVariable,
    },
    GreaterThan {
        out: EquivocMirVariable,
        lhs: EquivocMirVariable,
        rhs: EquivocMirVariable,
    },
    GreaterThanOrEquals {
        out: EquivocMirVariable,
        lhs: EquivocMirVariable,
        rhs: EquivocMirVariable,
    },
    LoadImage {
        out: EquivocMirVariable,
        path: EquivocMirVariable,
    },
    WriteImage {
        image: EquivocMirVariable,
        path: EquivocMirVariable,
    },
    ImageWidth {
        out: EquivocMirVariable,
        image: EquivocMirVariable,
    },
    ImageHeight {
        out: EquivocMirVariable,
        image: EquivocMirVariable,
    },
    ReadImagePixel {
        out: EquivocMirVariable,
        image: EquivocMirVariable,
        x: EquivocMirVariable,
        y: EquivocMirVariable,
    },
    WriteImagePixel {
        image: EquivocMirVariable,
        x: EquivocMirVariable,
        y: EquivocMirVariable,
        pixel: EquivocMirVariable,
    },
}
