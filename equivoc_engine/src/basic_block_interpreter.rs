use crate::lir::{
    EquivocLir, EquivocLirBasicBlock, EquivocLirBasicBlockId, EquivocLirInstruction,
    EquivocLirTerminateInstruction, EquivocLirValueType, EquivocLirVariable,
};
use std::any::Any;

struct Memory {
    memory: Vec<Option<Box<dyn Any>>>,
}

impl Memory {
    fn new() -> Self {
        Self { memory: Vec::new() }
    }

    fn get<T: ValidMemoryType>(&self, id: EquivocLirVariable) -> Option<&T> {
        self.memory
            .get(u32::from(id) as usize)
            .and_then(Option::as_ref)
            .and_then(|v| v.downcast_ref::<T>())
    }

    fn get_mut<T: ValidMemoryType>(&mut self, id: EquivocLirVariable) -> Option<&mut T> {
        self.memory
            .get_mut(u32::from(id) as usize)
            .and_then(Option::as_mut)
            .and_then(|v| v.downcast_mut::<T>())
    }

    fn set<T: ValidMemoryType>(&mut self, id: EquivocLirVariable, value: T) {
        let id = u32::from(id) as usize;
        let len = self.memory.len();
        if id >= len {
            self.memory.resize_with(id + 1, || None);
        }
        self.memory[id] = Some(Box::new(value));
    }
}

trait ValidMemoryType: Any + 'static {}

impl ValidMemoryType for i64 {}
impl ValidMemoryType for f64 {}
impl ValidMemoryType for String {}
impl ValidMemoryType for bool {}
impl ValidMemoryType for image::RgbaImage {}
impl ValidMemoryType for image::Rgba<u8> {}

pub fn execute(program: &EquivocLir) {
    let EquivocLir {
        basic_blocks,
        functions: _,
        entry_point,
    } = program;
    let mut memory = Memory::new();
    let mut pc = &basic_blocks[u32::from(*entry_point) as usize];

    execute_block(pc, &mut memory, program);
}

fn execute_block<'a>(
    mut pc: &'a EquivocLirBasicBlock,
    memory: &mut Memory,
    program: &'a EquivocLir,
) {
    let EquivocLir {
        basic_blocks,
        functions,
        entry_point: _,
    } = program;
    loop {
        let EquivocLirBasicBlock {
            id: _,
            instructions,
            terminate_instruction,
        } = pc;
        for i in instructions {
            match i {
                EquivocLirInstruction::LoadIntegerConst { out, value } => {
                    memory.set(*out, *value);
                }
                EquivocLirInstruction::LoadFloatConst { out, value } => {
                    memory.set(*out, *value);
                }
                EquivocLirInstruction::LoadStringConst { out, value } => {
                    memory.set(*out, value.clone());
                }
                EquivocLirInstruction::LoadBooleanConst { out, value } => {
                    memory.set(*out, *value);
                }
                EquivocLirInstruction::Assign { out, value } => {
                    macro_rules! assign {
                        ($ty:ty) => {{
                            let value = memory.get::<$ty>(*value).unwrap().clone();
                            memory.set(*out, value);
                        }};
                    }
                    match value.ty {
                        EquivocLirValueType::Integer => assign!(i64),
                        EquivocLirValueType::Float => assign!(f64),
                        EquivocLirValueType::String => assign!(String),
                        EquivocLirValueType::Boolean => assign!(bool),
                        EquivocLirValueType::Image => assign!(image::RgbaImage),
                        EquivocLirValueType::Pixel => assign!(image::Rgba<u8>),
                    }
                }
                EquivocLirInstruction::Add { out, lhs, rhs } => todo!(),
                EquivocLirInstruction::Sub { out, lhs, rhs } => todo!(),
                EquivocLirInstruction::Mul { out, lhs, rhs } => todo!(),
                EquivocLirInstruction::Div { out, lhs, rhs } => todo!(),
                EquivocLirInstruction::Mod { out, lhs, rhs } => todo!(),
                EquivocLirInstruction::Equals { out, lhs, rhs } => todo!(),
                EquivocLirInstruction::NotEquals { out, lhs, rhs } => todo!(),
                EquivocLirInstruction::LessThan { out, lhs, rhs } => todo!(),
                EquivocLirInstruction::LessThanOrEquals { out, lhs, rhs } => todo!(),
                EquivocLirInstruction::GreaterThan { out, lhs, rhs } => todo!(),
                EquivocLirInstruction::GreaterThanOrEquals { out, lhs, rhs } => todo!(),
                EquivocLirInstruction::WriteImage { image, path } => {
                    let image = memory.get::<image::RgbaImage>(*image).unwrap();
                    let path = memory.get::<String>(*path).unwrap();
                    image.save(&path).unwrap();
                }
                EquivocLirInstruction::ImageWidth { out, image } => {
                    let image = memory.get::<image::RgbaImage>(*image).unwrap();
                    let width = image.width();
                    memory.set(*out, width as i64);
                }
                EquivocLirInstruction::ImageHeight { out, image } => {
                    let image = memory.get::<image::RgbaImage>(*image).unwrap();
                    let height = image.height();
                    memory.set(*out, height as i64);
                }
                EquivocLirInstruction::ReadImagePixel { out, image, x, y } => {
                    let &x = memory.get::<i64>(*x).unwrap();
                    let &y = memory.get::<i64>(*y).unwrap();
                    let image = memory.get::<image::RgbaImage>(*image).unwrap();
                    let &pixel = image.get_pixel(x as u32, y as u32);
                    memory.set(*out, pixel);
                }
                EquivocLirInstruction::WriteImagePixel { image, x, y, pixel } => {
                    let &x = memory.get::<i64>(*x).unwrap();
                    let &y = memory.get::<i64>(*y).unwrap();
                    let &pixel = memory.get::<image::Rgba<u8>>(*pixel).unwrap();
                    let image = memory.get_mut::<image::RgbaImage>(*image).unwrap();
                    image.put_pixel(x as u32, y as u32, pixel);
                }
            }
        }
        match terminate_instruction {
            EquivocLirTerminateInstruction::Next { next_block } => {
                pc = &basic_blocks[u32::from(*next_block) as usize];
            }
            EquivocLirTerminateInstruction::If {
                condition,
                then_block,
                else_block,
            } => {
                let condition = memory.get(*condition).unwrap();
                if *condition {
                    pc = &basic_blocks[u32::from(*then_block) as usize];
                } else {
                    pc = &basic_blocks[u32::from(*else_block) as usize];
                }
            }
            EquivocLirTerminateInstruction::For {
                loop_count,
                loop_index,
                loop_block,
                next_block,
            } => {
                let len = *memory.get::<i64>(*loop_count).unwrap();
                let loop_block = &basic_blocks[u32::from(*loop_block) as usize];
                for i in 0..len {
                    memory.set(*loop_index, i);
                    execute_block(loop_block, memory, program);
                }
                pc = &basic_blocks[u32::from(*next_block) as usize];
            }
            EquivocLirTerminateInstruction::Continue => return,
            EquivocLirTerminateInstruction::Return { value } => return,
            EquivocLirTerminateInstruction::CallFunction {
                out,
                name,
                args,
                next,
            } => {
                todo!()
            }
            EquivocLirTerminateInstruction::LoadImage { out, path, next } => {
                let path = memory.get::<String>(*path).unwrap();
                let image = image::open(&path).unwrap().into_rgba8();
                memory.set(*out, image);
                pc = &basic_blocks[u32::from(*next) as usize];
            }
            EquivocLirTerminateInstruction::WaitForLoadImage { image, next } => {
                pc = &basic_blocks[u32::from(*next) as usize];
            }
        }
    }
}
