#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EquivocMirValueId(pub u32);

impl From<u32> for EquivocMirValueId {
    fn from(value: u32) -> Self {
        EquivocMirValueId(value)
    }
}

impl From<EquivocMirValueId> for u32 {
    fn from(value: EquivocMirValueId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EquivocMirLoopId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EquivocMirOperationId(pub u32);

#[derive(Debug)]
pub struct EquivocMir {
    pub values: Vec<EquivocMirValueData>,
    pub functions: Vec<EquivocMirFunction>,
    pub main_region: EquivocMirRegion,
}

#[derive(Debug)]
pub struct EquivocMirFunction {
    pub name: String,
    pub args: Vec<EquivocMirValueId>,
    pub body: EquivocMirRegion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquivocMirValueType {
    Unknown,
    Integer,
    Float,
    String,
    Boolean,
    Image(EquivocMirImageType),
    Pixel(EquivocMirImageDependentType),
    Sample(EquivocMirImageDependentType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EquivocMirImageType {
    pub header_source: EquivocMirValueId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EquivocMirImageDependentType {
    pub image: EquivocMirValueId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquivocMirValueDef {
    Unknown,
    FunctionArgument,
    OperationResult {
        operation: EquivocMirOperationId,
        result_index: u32,
    },
    RegionResult,
    LoopIndex {
        loop_id: EquivocMirLoopId,
    },
}

#[derive(Debug)]
pub struct EquivocMirValueData {
    pub ty: EquivocMirValueType,
    pub def: EquivocMirValueDef,
}

impl Default for EquivocMirValueData {
    fn default() -> Self {
        Self {
            ty: EquivocMirValueType::Unknown,
            def: EquivocMirValueDef::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct EquivocMirRegion {
    pub operations: Vec<EquivocMirOperation>,
    pub results: Vec<EquivocMirValueId>,
}

#[derive(Debug)]
pub struct EquivocMirOperation {
    pub id: EquivocMirOperationId,
    pub results: Vec<EquivocMirValueId>,
    pub kind: EquivocMirOperationKind,
    pub effects: EquivocMirEffectSummary,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct EquivocMirEffectSummary {
    pub reads: Vec<EquivocMirMemoryAccess>,
    pub writes: Vec<EquivocMirMemoryAccess>,
    pub ordered_effect: bool,
    pub control_effect: bool,
    pub irreversible_effect: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquivocMirMemoryAccess {
    pub resource: EquivocMirMemoryResource,
    pub region: EquivocMirMemoryRegion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquivocMirMemoryResource {
    Image { image: EquivocMirValueId },
    External,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquivocMirMemoryRegion {
    Whole,
    ImagePixel {
        x: EquivocMirValueId,
        y: EquivocMirValueId,
    },
    Unknown,
}

#[derive(Debug)]
pub enum EquivocMirOperationKind {
    If {
        condition: EquivocMirValueId,
        then_region: EquivocMirRegion,
        else_region: EquivocMirRegion,
    },
    For {
        loop_id: EquivocMirLoopId,
        count: EquivocMirValueId,
        index: EquivocMirValueId,
        carried: Vec<EquivocMirLoopCarried>,
        reductions: Vec<EquivocMirReduction>,
        body: EquivocMirRegion,
    },
    Loop {
        loop_id: EquivocMirLoopId,
        carried: Vec<EquivocMirLoopCarried>,
        body: EquivocMirRegion,
    },
    Break {
        target: EquivocMirLoopId,
    },
    Continue {
        target: EquivocMirLoopId,
    },
    Return {
        value: Option<EquivocMirValueId>,
    },
    CallFunction {
        name: String,
        args: Vec<EquivocMirValueId>,
    },
    LoadIntegerConst {
        value: i64,
    },
    LoadFloatConst {
        value: f64,
    },
    LoadStringConst {
        value: String,
    },
    LoadBooleanConst {
        value: bool,
    },
    Add {
        lhs: EquivocMirValueId,
        rhs: EquivocMirValueId,
    },
    Sub {
        lhs: EquivocMirValueId,
        rhs: EquivocMirValueId,
    },
    Mul {
        lhs: EquivocMirValueId,
        rhs: EquivocMirValueId,
    },
    Div {
        lhs: EquivocMirValueId,
        rhs: EquivocMirValueId,
    },
    Mod {
        lhs: EquivocMirValueId,
        rhs: EquivocMirValueId,
    },
    Equals {
        lhs: EquivocMirValueId,
        rhs: EquivocMirValueId,
    },
    NotEquals {
        lhs: EquivocMirValueId,
        rhs: EquivocMirValueId,
    },
    LessThan {
        lhs: EquivocMirValueId,
        rhs: EquivocMirValueId,
    },
    LessThanOrEquals {
        lhs: EquivocMirValueId,
        rhs: EquivocMirValueId,
    },
    GreaterThan {
        lhs: EquivocMirValueId,
        rhs: EquivocMirValueId,
    },
    GreaterThanOrEquals {
        lhs: EquivocMirValueId,
        rhs: EquivocMirValueId,
    },
    LoadImage {
        path: EquivocMirValueId,
    },
    WriteImage {
        image: EquivocMirValueId,
        path: EquivocMirValueId,
    },
    ImageWidth {
        image: EquivocMirValueId,
    },
    ImageHeight {
        image: EquivocMirValueId,
    },
    ReadImagePixel {
        image: EquivocMirValueId,
        x: EquivocMirValueId,
        y: EquivocMirValueId,
    },
    WriteImagePixel {
        image: EquivocMirValueId,
        x: EquivocMirValueId,
        y: EquivocMirValueId,
        pixel: EquivocMirValueId,
    },
}

#[derive(Debug)]
pub struct EquivocMirLoopCarried {
    pub initial: EquivocMirValueId,
    pub body_result: EquivocMirValueId,
    pub result: EquivocMirValueId,
}

#[derive(Debug)]
pub struct EquivocMirReduction {
    pub initial: EquivocMirValueId,
    pub accumulator: EquivocMirValueId,
    pub reduced_value: EquivocMirValueId,
    pub result: EquivocMirValueId,
    pub op: EquivocMirReductionOp,
    pub ordering: EquivocMirReductionOrdering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquivocMirReductionOp {
    Add,
    Mul,
    Min,
    Max,
    BitAnd,
    BitOr,
    BitXor,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquivocMirReductionOrdering {
    Strict,
    DeterministicTree,
    Reassociate,
}
