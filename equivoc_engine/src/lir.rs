#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquivocLirValueType {
    Integer,
    Float,
    String,
    Boolean,
    Image,
    Pixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EquivocLirVariable {
    pub id: u32,
    pub ty: EquivocLirValueType,
}

impl From<EquivocLirVariable> for u32 {
    fn from(value: EquivocLirVariable) -> Self {
        value.id
    }
}

#[derive(Debug)]
pub struct EquivocLir {
    pub basic_blocks: Vec<EquivocLirBasicBlock>,
    pub functions: Vec<EquivocLirFunction>,
    pub entry_point: EquivocLirBasicBlockId,
}

#[derive(Debug)]
pub struct EquivocLirFunction {
    pub name: String,
    pub args: Vec<EquivocLirVariable>,
    pub entry_point: EquivocLirBasicBlockId,
}

#[derive(Debug)]
pub struct VariableUpdate {
    pub variable: EquivocLirVariable,
    pub then_variable: EquivocLirVariable,
    pub else_variable: EquivocLirVariable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EquivocLirBasicBlockId(u32);

impl From<u32> for EquivocLirBasicBlockId {
    fn from(value: u32) -> Self {
        EquivocLirBasicBlockId(value)
    }
}

impl From<EquivocLirBasicBlockId> for u32 {
    fn from(value: EquivocLirBasicBlockId) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub struct EquivocLirBasicBlock {
    pub id: EquivocLirBasicBlockId,
    pub instructions: Vec<EquivocLirInstruction>,
    pub terminate_instruction: EquivocLirTerminateInstruction,
}

pub struct EquivocLirBasicBlockBuilder {
    id: EquivocLirBasicBlockId,
    instructions: Vec<EquivocLirInstruction>,
}

impl EquivocLirBasicBlockBuilder {
    fn new(id: EquivocLirBasicBlockId) -> Self {
        EquivocLirBasicBlockBuilder {
            id,
            instructions: Vec::new(),
        }
    }

    pub fn id(&self) -> EquivocLirBasicBlockId {
        self.id
    }

    pub fn add_instruction(&mut self, instruction: EquivocLirInstruction) {
        self.instructions.push(instruction);
    }

    pub fn finish(
        self,
        terminate_instruction: EquivocLirTerminateInstruction,
    ) -> EquivocLirBasicBlock {
        EquivocLirBasicBlock {
            id: self.id,
            instructions: self.instructions,
            terminate_instruction,
        }
    }
}

pub struct EquivocLirBuilder {
    basic_blocks: Vec<Option<EquivocLirBasicBlock>>,
    functions: Vec<EquivocLirFunction>,
}

impl EquivocLirBuilder {
    pub fn new() -> Self {
        EquivocLirBuilder {
            basic_blocks: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn next_block(&mut self) -> EquivocLirBasicBlockBuilder {
        let id = u32::try_from(self.basic_blocks.len()).unwrap();
        self.basic_blocks.push(None);
        EquivocLirBasicBlockBuilder::new(EquivocLirBasicBlockId(id))
    }

    pub fn add_basic_block(&mut self, basic_block: EquivocLirBasicBlock) {
        let index = basic_block.id.0 as usize;
        assert!(self.basic_blocks[index].replace(basic_block).is_none());
    }

    pub fn add_function(&mut self, function: EquivocLirFunction) {
        self.functions.push(function);
    }

    pub fn finish(self, entry_point: EquivocLirBasicBlockId) -> EquivocLir {
        let EquivocLirBuilder {
            basic_blocks,
            functions,
        } = self;
        EquivocLir {
            basic_blocks: basic_blocks.into_iter().map(Option::unwrap).collect(),
            functions,
            entry_point,
        }
    }
}

#[derive(Debug)]
pub enum EquivocLirInstruction {
    LoadIntegerConst {
        out: EquivocLirVariable,
        value: i64,
    },
    LoadFloatConst {
        out: EquivocLirVariable,
        value: f64,
    },
    LoadStringConst {
        out: EquivocLirVariable,
        value: String,
    },
    LoadBooleanConst {
        out: EquivocLirVariable,
        value: bool,
    },
    Assign {
        out: EquivocLirVariable,
        value: EquivocLirVariable,
    },
    Add {
        out: EquivocLirVariable,
        lhs: EquivocLirVariable,
        rhs: EquivocLirVariable,
    },
    Sub {
        out: EquivocLirVariable,
        lhs: EquivocLirVariable,
        rhs: EquivocLirVariable,
    },
    Mul {
        out: EquivocLirVariable,
        lhs: EquivocLirVariable,
        rhs: EquivocLirVariable,
    },
    Div {
        out: EquivocLirVariable,
        lhs: EquivocLirVariable,
        rhs: EquivocLirVariable,
    },
    Mod {
        out: EquivocLirVariable,
        lhs: EquivocLirVariable,
        rhs: EquivocLirVariable,
    },
    Equals {
        out: EquivocLirVariable,
        lhs: EquivocLirVariable,
        rhs: EquivocLirVariable,
    },
    NotEquals {
        out: EquivocLirVariable,
        lhs: EquivocLirVariable,
        rhs: EquivocLirVariable,
    },
    LessThan {
        out: EquivocLirVariable,
        lhs: EquivocLirVariable,
        rhs: EquivocLirVariable,
    },
    LessThanOrEquals {
        out: EquivocLirVariable,
        lhs: EquivocLirVariable,
        rhs: EquivocLirVariable,
    },
    GreaterThan {
        out: EquivocLirVariable,
        lhs: EquivocLirVariable,
        rhs: EquivocLirVariable,
    },
    GreaterThanOrEquals {
        out: EquivocLirVariable,
        lhs: EquivocLirVariable,
        rhs: EquivocLirVariable,
    },
    WriteImage {
        image: EquivocLirVariable,
        path: EquivocLirVariable,
    },
    ImageWidth {
        out: EquivocLirVariable,
        image: EquivocLirVariable,
    },
    ImageHeight {
        out: EquivocLirVariable,
        image: EquivocLirVariable,
    },
    ReadImagePixel {
        out: EquivocLirVariable,
        image: EquivocLirVariable,
        x: EquivocLirVariable,
        y: EquivocLirVariable,
    },
    WriteImagePixel {
        image: EquivocLirVariable,
        x: EquivocLirVariable,
        y: EquivocLirVariable,
        pixel: EquivocLirVariable,
    },
}

#[derive(Debug)]
pub enum EquivocLirTerminateInstruction {
    Next {
        next_block: EquivocLirBasicBlockId,
    },
    If {
        condition: EquivocLirVariable,
        then_block: EquivocLirBasicBlockId,
        else_block: EquivocLirBasicBlockId,
    },
    For {
        loop_count: EquivocLirVariable,
        loop_index: EquivocLirVariable,
        loop_block: EquivocLirBasicBlockId,
        next_block: EquivocLirBasicBlockId,
    },
    Continue,
    Return {
        value: Option<EquivocLirVariable>,
    },
    CallFunction {
        out: Option<EquivocLirVariable>,
        name: String,
        args: Vec<EquivocLirVariable>,
        next: EquivocLirBasicBlockId,
    },
    LoadImage {
        out: EquivocLirVariable,
        path: EquivocLirVariable,
        next: EquivocLirBasicBlockId,
    },
    WaitForLoadImage {
        image: EquivocLirVariable,
        next: EquivocLirBasicBlockId,
    },
}
