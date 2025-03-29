use crate::lir::{
    EquivocLir, EquivocLirBasicBlock, EquivocLirBasicBlockBuilder, EquivocLirBasicBlockId,
    EquivocLirBuilder, EquivocLirFunction, EquivocLirInstruction, EquivocLirTerminateInstruction,
    EquivocLirValueType, EquivocLirVariable,
};
use crate::mir::{
    EquivocMir, EquivocMirInstruction, EquivocMirVariable, IfVariableUpdate, LoopVariableUpdate,
};
use std::collections::HashMap;

type TypeMap = HashMap<EquivocMirVariable, EquivocLirValueType>;

/// MIR命令を解析して変数の型を推論する（再帰的）
fn infer_types(instructions: &[EquivocMirInstruction], type_map: &mut TypeMap) {
    for instr in instructions {
        match instr {
            // Load ～ 系は出力変数の型を直接決定できる
            EquivocMirInstruction::LoadIntegerConst { out, .. } => {
                type_map.insert(*out, EquivocLirValueType::Integer);
            }
            EquivocMirInstruction::LoadFloatConst { out, .. } => {
                type_map.insert(*out, EquivocLirValueType::Float);
            }
            EquivocMirInstruction::LoadStringConst { out, .. } => {
                type_map.insert(*out, EquivocLirValueType::String);
            }
            EquivocMirInstruction::LoadBooleanConst { out, .. } => {
                type_map.insert(*out, EquivocLirValueType::Boolean);
            }
            EquivocMirInstruction::LoadImage { out, .. } => {
                type_map.insert(*out, EquivocLirValueType::Image);
            }

            EquivocMirInstruction::Add { out, lhs, rhs }
            | EquivocMirInstruction::Sub { out, lhs, rhs }
            | EquivocMirInstruction::Mul { out, lhs, rhs }
            | EquivocMirInstruction::Div { out, lhs, rhs }
            | EquivocMirInstruction::Mod { out, lhs, rhs } => {
                let lhs_type = type_map.get(lhs).unwrap();
                assert_eq!(lhs_type, type_map.get(rhs).unwrap());
                type_map.insert(*out, lhs_type.clone());
            }
            EquivocMirInstruction::Equals { out, lhs, rhs }
            | EquivocMirInstruction::NotEquals { out, lhs, rhs }
            | EquivocMirInstruction::LessThan { out, lhs, rhs }
            | EquivocMirInstruction::LessThanOrEquals { out, lhs, rhs }
            | EquivocMirInstruction::GreaterThan { out, lhs, rhs }
            | EquivocMirInstruction::GreaterThanOrEquals { out, lhs, rhs } => {
                let lhs_type = type_map.get(lhs).unwrap();
                assert_eq!(lhs_type, type_map.get(rhs).unwrap());
                type_map.insert(*out, EquivocLirValueType::Boolean);
            }
            EquivocMirInstruction::ImageWidth { out, image }
            | EquivocMirInstruction::ImageHeight { out, image } => {
                assert_eq!(type_map.get(image).unwrap(), &EquivocLirValueType::Image);
                type_map.insert(*out, EquivocLirValueType::Integer);
            }
            EquivocMirInstruction::ReadImagePixel { out, image, x, y } => {
                assert_eq!(type_map.get(image).unwrap(), &EquivocLirValueType::Image);
                assert_eq!(type_map.get(x).unwrap(), &EquivocLirValueType::Integer);
                assert_eq!(type_map.get(y).unwrap(), &EquivocLirValueType::Integer);
                type_map.insert(*out, EquivocLirValueType::Pixel);
            }
            EquivocMirInstruction::WriteImage { image, path } => {
                assert_eq!(type_map.get(image).unwrap(), &EquivocLirValueType::Image);
                assert_eq!(type_map.get(path).unwrap(), &EquivocLirValueType::String);
            }
            EquivocMirInstruction::WriteImagePixel { image, x, y, pixel } => {
                assert_eq!(type_map.get(image).unwrap(), &EquivocLirValueType::Image);
                assert_eq!(type_map.get(x).unwrap(), &EquivocLirValueType::Integer);
                assert_eq!(type_map.get(y).unwrap(), &EquivocLirValueType::Integer);
                assert_eq!(type_map.get(pixel).unwrap(), &EquivocLirValueType::Pixel);
            }

            EquivocMirInstruction::CallFunction { .. } => {
                todo!()
            }
            EquivocMirInstruction::Return { .. } => {
                todo!()
            }

            EquivocMirInstruction::If {
                condition,
                then_instructions,
                else_instructions,
                ..
            } => {
                assert_eq!(
                    type_map.get(condition).unwrap(),
                    &EquivocLirValueType::Boolean
                );
                infer_types(then_instructions, type_map);
                infer_types(else_instructions, type_map);
            }
            EquivocMirInstruction::For {
                variable_updates,
                loop_counts,
                loop_indices,
                instructions,
            } => {
                for i in loop_counts {
                    assert_eq!(type_map.get(i).unwrap(), &EquivocLirValueType::Integer);
                }
                for i in loop_indices {
                    type_map.insert(*i, EquivocLirValueType::Integer);
                }
                infer_types(instructions, type_map);
                for v in variable_updates {
                    assert_eq!(
                        type_map.get(&v.base).unwrap(),
                        type_map.get(&v.updated).unwrap()
                    );
                }
            }
            EquivocMirInstruction::Loop {
                variable_updates,
                instructions,
            } => {
                infer_types(instructions, type_map);
                for v in variable_updates {
                    assert_eq!(
                        type_map.get(&v.base).unwrap(),
                        type_map.get(&v.updated).unwrap()
                    );
                }
            }

            EquivocMirInstruction::Break | EquivocMirInstruction::Continue => {}
        }
    }
}

fn to_lir_var(id: EquivocMirVariable, type_map: &TypeMap) -> EquivocLirVariable {
    let ty = type_map.get(&id).unwrap().clone();
    EquivocLirVariable {
        id: u32::from(id),
        ty,
    }
}

fn build_single_block_from_instructions(
    instructions: &[EquivocMirInstruction],
    type_map: &TypeMap,
    lir_builder: &mut EquivocLirBuilder,
    mut block_builder: EquivocLirBasicBlockBuilder,
    finish_block: impl FnOnce(EquivocLirBasicBlockBuilder) -> EquivocLirBasicBlock,
) {
    macro_rules! refresh_block {
        ($id:ident => $terminate_instruction:expr) => {
            let next_block_builder = lir_builder.next_block();
            let $id = next_block_builder.id();
            let block = block_builder.finish($terminate_instruction);
            lir_builder.add_basic_block(block);
            block_builder = next_block_builder;
        };
        ($terminate_instruction:expr) => {
            refresh_block!(_ => $terminate_instruction);
        }
    }
    for instr in instructions {
        match instr {
            EquivocMirInstruction::LoadIntegerConst { out, value } => {
                block_builder.add_instruction(EquivocLirInstruction::LoadIntegerConst {
                    out: to_lir_var(*out, type_map),
                    value: *value,
                });
            }
            EquivocMirInstruction::LoadFloatConst { out, value } => {
                block_builder.add_instruction(EquivocLirInstruction::LoadFloatConst {
                    out: to_lir_var(*out, type_map),
                    value: *value,
                });
            }
            EquivocMirInstruction::LoadStringConst { out, value } => {
                block_builder.add_instruction(EquivocLirInstruction::LoadStringConst {
                    out: to_lir_var(*out, type_map),
                    value: value.clone(),
                });
            }
            EquivocMirInstruction::LoadBooleanConst { out, value } => {
                block_builder.add_instruction(EquivocLirInstruction::LoadBooleanConst {
                    out: to_lir_var(*out, type_map),
                    value: *value,
                });
            }
            EquivocMirInstruction::Add { out, lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Add {
                    out: to_lir_var(*out, type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirInstruction::Sub { out, lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Sub {
                    out: to_lir_var(*out, type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirInstruction::Mul { out, lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Mul {
                    out: to_lir_var(*out, type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirInstruction::Div { out, lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Div {
                    out: to_lir_var(*out, type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirInstruction::Mod { out, lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Mod {
                    out: to_lir_var(*out, type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirInstruction::Equals { out, lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Equals {
                    out: to_lir_var(*out, type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirInstruction::NotEquals { out, lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::NotEquals {
                    out: to_lir_var(*out, type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirInstruction::LessThan { out, lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::LessThan {
                    out: to_lir_var(*out, type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirInstruction::LessThanOrEquals { out, lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::LessThanOrEquals {
                    out: to_lir_var(*out, type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirInstruction::GreaterThan { out, lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::GreaterThan {
                    out: to_lir_var(*out, type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirInstruction::GreaterThanOrEquals { out, lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::GreaterThanOrEquals {
                    out: to_lir_var(*out, type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirInstruction::WriteImage { image, path } => {
                block_builder.add_instruction(EquivocLirInstruction::WriteImage {
                    image: to_lir_var(*image, type_map),
                    path: to_lir_var(*path, type_map),
                });
            }
            EquivocMirInstruction::ImageWidth { out, image } => {
                block_builder.add_instruction(EquivocLirInstruction::ImageWidth {
                    out: to_lir_var(*out, type_map),
                    image: to_lir_var(*image, type_map),
                });
            }
            EquivocMirInstruction::ImageHeight { out, image } => {
                block_builder.add_instruction(EquivocLirInstruction::ImageHeight {
                    out: to_lir_var(*out, type_map),
                    image: to_lir_var(*image, type_map),
                });
            }
            EquivocMirInstruction::ReadImagePixel { out, image, x, y } => {
                block_builder.add_instruction(EquivocLirInstruction::ReadImagePixel {
                    out: to_lir_var(*out, type_map),
                    image: to_lir_var(*image, type_map),
                    x: to_lir_var(*x, type_map),
                    y: to_lir_var(*y, type_map),
                });
            }
            EquivocMirInstruction::WriteImagePixel { image, x, y, pixel } => {
                block_builder.add_instruction(EquivocLirInstruction::WriteImagePixel {
                    image: to_lir_var(*image, type_map),
                    x: to_lir_var(*x, type_map),
                    y: to_lir_var(*y, type_map),
                    pixel: to_lir_var(*pixel, type_map),
                });
            }

            EquivocMirInstruction::LoadImage { out, path } => {
                let out = to_lir_var(*out, type_map);
                refresh_block!(next_id => EquivocLirTerminateInstruction::LoadImage {
                    out,
                    path: to_lir_var(*path, type_map),
                    next: next_id,
                });
                refresh_block!(next_id => EquivocLirTerminateInstruction::WaitForLoadImage { image: out, next: next_id });
            }
            EquivocMirInstruction::CallFunction { out, name, args } => {
                refresh_block!(next_id => EquivocLirTerminateInstruction::CallFunction {
                    out: out.map(|out| to_lir_var(out, type_map)),
                    name: name.clone(),
                    args: args.iter().map(|arg| to_lir_var(*arg, type_map)).collect(),
                    next: next_id,
                });
            }
            EquivocMirInstruction::Return { value } => {
                let block = block_builder.finish(EquivocLirTerminateInstruction::Return {
                    value: value.map(|value| to_lir_var(value, type_map)),
                });
                lir_builder.add_basic_block(block);
                return;
            }
            EquivocMirInstruction::If {
                variables,
                condition,
                then_instructions,
                else_instructions,
            } => {
                let then_block = lir_builder.next_block();
                let else_block = lir_builder.next_block();
                refresh_block!(_next_block => EquivocLirTerminateInstruction::If {
                    condition: to_lir_var(*condition, type_map),
                    then_block: then_block.id(),
                    else_block: else_block.id(),
                });
                let next_id = block_builder.id();
                fn finish_then_block(
                    variables: &[IfVariableUpdate],
                    next_id: EquivocLirBasicBlockId,
                    type_map: &TypeMap,
                ) -> impl FnOnce(EquivocLirBasicBlockBuilder) -> EquivocLirBasicBlock
                {
                    move |mut block_builder| {
                        for v in variables {
                            block_builder.add_instruction(EquivocLirInstruction::Assign {
                                out: to_lir_var(v.variable, type_map),
                                value: to_lir_var(v.then_variable, type_map),
                            });
                        }
                        block_builder.finish(EquivocLirTerminateInstruction::Next {
                            next_block: next_id,
                        })
                    }
                }
                fn finish_else_block(
                    variables: &[IfVariableUpdate],
                    next_id: EquivocLirBasicBlockId,
                    type_map: &TypeMap,
                ) -> impl FnOnce(EquivocLirBasicBlockBuilder) -> EquivocLirBasicBlock
                {
                    move |mut block_builder| {
                        for v in variables {
                            block_builder.add_instruction(EquivocLirInstruction::Assign {
                                out: to_lir_var(v.variable, type_map),
                                value: to_lir_var(v.else_variable, type_map),
                            });
                        }
                        block_builder.finish(EquivocLirTerminateInstruction::Next {
                            next_block: next_id,
                        })
                    }
                }
                build_single_block_from_instructions(
                    then_instructions,
                    type_map,
                    lir_builder,
                    then_block,
                    finish_then_block(&variables, next_id, type_map),
                );
                build_single_block_from_instructions(
                    else_instructions,
                    type_map,
                    lir_builder,
                    else_block,
                    finish_else_block(&variables, next_id, type_map),
                );
            }
            EquivocMirInstruction::For {
                variable_updates,
                loop_counts,
                loop_indices,
                instructions,
            } => {
                let for_block = lir_builder.next_block();
                refresh_block!(next_block => EquivocLirTerminateInstruction::For {
                    loop_counts: loop_counts.iter().map(|v| to_lir_var(*v, type_map)).collect(),
                    loop_indices: loop_indices.iter().map(|v| to_lir_var(*v, type_map)).collect(),
                    loop_block: for_block.id(),
                    next_block,
                });
                fn finish_for(
                    variable_updates: &[LoopVariableUpdate],
                    type_map: &TypeMap,
                ) -> impl FnOnce(EquivocLirBasicBlockBuilder) -> EquivocLirBasicBlock
                {
                    move |mut block_builder| {
                        for v in variable_updates {
                            block_builder.add_instruction(EquivocLirInstruction::Assign {
                                out: to_lir_var(v.base, type_map),
                                value: to_lir_var(v.updated, type_map),
                            });
                        }
                        block_builder.finish(EquivocLirTerminateInstruction::Continue)
                    }
                }
                build_single_block_from_instructions(
                    instructions,
                    type_map,
                    lir_builder,
                    for_block,
                    finish_for(&variable_updates, type_map),
                );
            }
            EquivocMirInstruction::Loop { .. }
            | EquivocMirInstruction::Break
            | EquivocMirInstruction::Continue => {
                todo!()
            }
        }
    }
    let block = finish_block(block_builder);
    lir_builder.add_basic_block(block);
}

pub fn convert_equivoc_mir_to_equivoc_lir(mir: &EquivocMir) -> EquivocLir {
    let mut type_map = TypeMap::new();

    for func in &mir.functions {
        infer_types(&func.instructions, &mut type_map);
    }
    infer_types(&mir.main_instructions, &mut type_map);

    let mut lir_builder = EquivocLirBuilder::new();

    for func in &mir.functions {
        let lir_args: Vec<_> = func
            .args
            .iter()
            .map(|mvar| to_lir_var(*mvar, &type_map))
            .collect();

        let entry_block = lir_builder.next_block();
        let entry_id = entry_block.id();
        build_single_block_from_instructions(
            &func.instructions,
            &type_map,
            &mut lir_builder,
            entry_block,
            |block_builder| {
                block_builder.finish(EquivocLirTerminateInstruction::Return { value: None })
            },
        );

        lir_builder.add_function(EquivocLirFunction {
            name: func.name.clone(),
            args: lir_args,
            entry_point: entry_id,
        });
    }

    let entry_block = lir_builder.next_block();
    let entry_id = entry_block.id();
    build_single_block_from_instructions(
        &mir.main_instructions,
        &type_map,
        &mut lir_builder,
        entry_block,
        |block_builder| {
            block_builder.finish(EquivocLirTerminateInstruction::Return { value: None })
        },
    );

    lir_builder.finish(entry_id)
}
