use crate::lir::{
    EquivocLir, EquivocLirBasicBlock, EquivocLirBasicBlockBuilder, EquivocLirBuilder,
    EquivocLirFunction, EquivocLirInstruction, EquivocLirTerminateInstruction, EquivocLirValueType,
    EquivocLirVariable,
};
use crate::mir::{
    EquivocMir, EquivocMirLoopCarried, EquivocMirOperation, EquivocMirOperationKind,
    EquivocMirReduction, EquivocMirReductionOp, EquivocMirRegion, EquivocMirValueId,
};
use std::collections::HashMap;

type TypeMap = HashMap<EquivocMirValueId, EquivocLirValueType>;

fn first_result(op: &EquivocMirOperation) -> EquivocMirValueId {
    *op.results
        .first()
        .expect("operation must have a result value")
}

fn infer_region_types(region: &EquivocMirRegion, type_map: &mut TypeMap) {
    for op in &region.operations {
        match &op.kind {
            EquivocMirOperationKind::LoadIntegerConst { .. } => {
                type_map.insert(first_result(op), EquivocLirValueType::Integer);
            }
            EquivocMirOperationKind::LoadFloatConst { .. } => {
                type_map.insert(first_result(op), EquivocLirValueType::Float);
            }
            EquivocMirOperationKind::LoadStringConst { .. } => {
                type_map.insert(first_result(op), EquivocLirValueType::String);
            }
            EquivocMirOperationKind::LoadBooleanConst { .. } => {
                type_map.insert(first_result(op), EquivocLirValueType::Boolean);
            }
            EquivocMirOperationKind::LoadImage { .. } => {
                type_map.insert(first_result(op), EquivocLirValueType::Image);
            }
            EquivocMirOperationKind::Add { lhs, rhs }
            | EquivocMirOperationKind::Sub { lhs, rhs }
            | EquivocMirOperationKind::Mul { lhs, rhs }
            | EquivocMirOperationKind::Div { lhs, rhs }
            | EquivocMirOperationKind::Mod { lhs, rhs } => {
                let lhs_type = *type_map.get(lhs).unwrap();
                assert_eq!(&lhs_type, type_map.get(rhs).unwrap());
                type_map.insert(first_result(op), lhs_type);
            }
            EquivocMirOperationKind::Equals { lhs, rhs }
            | EquivocMirOperationKind::NotEquals { lhs, rhs }
            | EquivocMirOperationKind::LessThan { lhs, rhs }
            | EquivocMirOperationKind::LessThanOrEquals { lhs, rhs }
            | EquivocMirOperationKind::GreaterThan { lhs, rhs }
            | EquivocMirOperationKind::GreaterThanOrEquals { lhs, rhs } => {
                let lhs_type = type_map.get(lhs).unwrap();
                assert_eq!(lhs_type, type_map.get(rhs).unwrap());
                type_map.insert(first_result(op), EquivocLirValueType::Boolean);
            }
            EquivocMirOperationKind::ImageWidth { image }
            | EquivocMirOperationKind::ImageHeight { image } => {
                assert_eq!(type_map.get(image).unwrap(), &EquivocLirValueType::Image);
                type_map.insert(first_result(op), EquivocLirValueType::Integer);
            }
            EquivocMirOperationKind::ReadImagePixel { image, x, y } => {
                assert_eq!(type_map.get(image).unwrap(), &EquivocLirValueType::Image);
                assert_eq!(type_map.get(x).unwrap(), &EquivocLirValueType::Integer);
                assert_eq!(type_map.get(y).unwrap(), &EquivocLirValueType::Integer);
                type_map.insert(first_result(op), EquivocLirValueType::Pixel);
            }
            EquivocMirOperationKind::WriteImage { image, path } => {
                assert_eq!(type_map.get(image).unwrap(), &EquivocLirValueType::Image);
                assert_eq!(type_map.get(path).unwrap(), &EquivocLirValueType::String);
            }
            EquivocMirOperationKind::WriteImagePixel { image, x, y, pixel } => {
                assert_eq!(type_map.get(image).unwrap(), &EquivocLirValueType::Image);
                assert_eq!(type_map.get(x).unwrap(), &EquivocLirValueType::Integer);
                assert_eq!(type_map.get(y).unwrap(), &EquivocLirValueType::Integer);
                assert_eq!(type_map.get(pixel).unwrap(), &EquivocLirValueType::Pixel);
            }
            EquivocMirOperationKind::If {
                condition,
                then_region,
                else_region,
            } => {
                assert_eq!(
                    type_map.get(condition).unwrap(),
                    &EquivocLirValueType::Boolean
                );
                infer_region_types(then_region, type_map);
                infer_region_types(else_region, type_map);
                assert_eq!(op.results.len(), then_region.results.len());
                assert_eq!(op.results.len(), else_region.results.len());
                for ((result, then_value), else_value) in op
                    .results
                    .iter()
                    .zip(&then_region.results)
                    .zip(&else_region.results)
                {
                    let ty = *type_map.get(then_value).unwrap();
                    assert_eq!(&ty, type_map.get(else_value).unwrap());
                    type_map.insert(*result, ty);
                }
            }
            EquivocMirOperationKind::For {
                count,
                index,
                carried,
                reductions,
                body,
                ..
            } => {
                assert_eq!(type_map.get(count).unwrap(), &EquivocLirValueType::Integer);
                type_map.insert(*index, EquivocLirValueType::Integer);
                infer_region_types(body, type_map);
                assert_eq!(op.results.len(), carried.len() + reductions.len());
                for (result, carried) in op.results.iter().zip(carried) {
                    let ty = *type_map.get(&carried.initial).unwrap();
                    assert_eq!(&ty, type_map.get(&carried.body_result).unwrap());
                    type_map.insert(*result, ty);
                }
                infer_reduction_types(reductions, type_map);
            }
            EquivocMirOperationKind::Loop { body, carried, .. } => {
                infer_region_types(body, type_map);
                assert_eq!(op.results.len(), carried.len());
                for (result, carried) in op.results.iter().zip(carried) {
                    let ty = *type_map.get(&carried.initial).unwrap();
                    assert_eq!(&ty, type_map.get(&carried.body_result).unwrap());
                    type_map.insert(*result, ty);
                }
            }
            EquivocMirOperationKind::CallFunction { .. }
            | EquivocMirOperationKind::Return { .. } => {
                todo!()
            }
            EquivocMirOperationKind::Break { .. } | EquivocMirOperationKind::Continue { .. } => {}
        }
    }
}

fn to_lir_var(id: EquivocMirValueId, type_map: &TypeMap) -> EquivocLirVariable {
    EquivocLirVariable {
        id: u32::from(id),
        ty: *type_map.get(&id).unwrap(),
    }
}

fn add_region_result_assigns(
    block_builder: &mut EquivocLirBasicBlockBuilder,
    results: &[EquivocMirValueId],
    values: &[EquivocMirValueId],
    type_map: &TypeMap,
) {
    assert_eq!(results.len(), values.len());
    for (result, value) in results.iter().zip(values) {
        block_builder.add_instruction(EquivocLirInstruction::Assign {
            out: to_lir_var(*result, type_map),
            value: to_lir_var(*value, type_map),
        });
    }
}

fn add_loop_carried_assigns(
    block_builder: &mut EquivocLirBasicBlockBuilder,
    carried: &[EquivocMirLoopCarried],
    type_map: &TypeMap,
) {
    for carried in carried {
        block_builder.add_instruction(EquivocLirInstruction::Assign {
            out: to_lir_var(carried.initial, type_map),
            value: to_lir_var(carried.body_result, type_map),
        });
    }
}

fn infer_reduction_types(reductions: &[EquivocMirReduction], type_map: &mut TypeMap) {
    for reduction in reductions {
        let ty = *type_map.get(&reduction.initial).unwrap();
        assert_eq!(&ty, type_map.get(&reduction.accumulator).unwrap());
        assert_eq!(&ty, type_map.get(&reduction.reduced_value).unwrap());
        match reduction.op {
            EquivocMirReductionOp::LogicalAnd | EquivocMirReductionOp::LogicalOr => {
                assert_eq!(ty, EquivocLirValueType::Boolean);
            }
            EquivocMirReductionOp::BitAnd
            | EquivocMirReductionOp::BitOr
            | EquivocMirReductionOp::BitXor => {
                assert_eq!(ty, EquivocLirValueType::Integer);
            }
            EquivocMirReductionOp::Add
            | EquivocMirReductionOp::Mul
            | EquivocMirReductionOp::Min
            | EquivocMirReductionOp::Max => {}
        }
        type_map.insert(reduction.result, ty);
    }
}

fn build_blocks_from_region(
    region: &EquivocMirRegion,
    type_map: &TypeMap,
    lir_builder: &mut EquivocLirBuilder,
    mut block_builder: EquivocLirBasicBlockBuilder,
    finish_block: Box<dyn FnOnce(EquivocLirBasicBlockBuilder) -> EquivocLirBasicBlock + '_>,
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
        };
    }

    for op in &region.operations {
        match &op.kind {
            EquivocMirOperationKind::LoadIntegerConst { value } => {
                block_builder.add_instruction(EquivocLirInstruction::LoadIntegerConst {
                    out: to_lir_var(first_result(op), type_map),
                    value: *value,
                });
            }
            EquivocMirOperationKind::LoadFloatConst { value } => {
                block_builder.add_instruction(EquivocLirInstruction::LoadFloatConst {
                    out: to_lir_var(first_result(op), type_map),
                    value: *value,
                });
            }
            EquivocMirOperationKind::LoadStringConst { value } => {
                block_builder.add_instruction(EquivocLirInstruction::LoadStringConst {
                    out: to_lir_var(first_result(op), type_map),
                    value: value.clone(),
                });
            }
            EquivocMirOperationKind::LoadBooleanConst { value } => {
                block_builder.add_instruction(EquivocLirInstruction::LoadBooleanConst {
                    out: to_lir_var(first_result(op), type_map),
                    value: *value,
                });
            }
            EquivocMirOperationKind::Add { lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Add {
                    out: to_lir_var(first_result(op), type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirOperationKind::Sub { lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Sub {
                    out: to_lir_var(first_result(op), type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirOperationKind::Mul { lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Mul {
                    out: to_lir_var(first_result(op), type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirOperationKind::Div { lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Div {
                    out: to_lir_var(first_result(op), type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirOperationKind::Mod { lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Mod {
                    out: to_lir_var(first_result(op), type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirOperationKind::Equals { lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::Equals {
                    out: to_lir_var(first_result(op), type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirOperationKind::NotEquals { lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::NotEquals {
                    out: to_lir_var(first_result(op), type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirOperationKind::LessThan { lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::LessThan {
                    out: to_lir_var(first_result(op), type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirOperationKind::LessThanOrEquals { lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::LessThanOrEquals {
                    out: to_lir_var(first_result(op), type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirOperationKind::GreaterThan { lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::GreaterThan {
                    out: to_lir_var(first_result(op), type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirOperationKind::GreaterThanOrEquals { lhs, rhs } => {
                block_builder.add_instruction(EquivocLirInstruction::GreaterThanOrEquals {
                    out: to_lir_var(first_result(op), type_map),
                    lhs: to_lir_var(*lhs, type_map),
                    rhs: to_lir_var(*rhs, type_map),
                });
            }
            EquivocMirOperationKind::WriteImage { image, path } => {
                block_builder.add_instruction(EquivocLirInstruction::WriteImage {
                    image: to_lir_var(*image, type_map),
                    path: to_lir_var(*path, type_map),
                });
            }
            EquivocMirOperationKind::ImageWidth { image } => {
                block_builder.add_instruction(EquivocLirInstruction::ImageWidth {
                    out: to_lir_var(first_result(op), type_map),
                    image: to_lir_var(*image, type_map),
                });
            }
            EquivocMirOperationKind::ImageHeight { image } => {
                block_builder.add_instruction(EquivocLirInstruction::ImageHeight {
                    out: to_lir_var(first_result(op), type_map),
                    image: to_lir_var(*image, type_map),
                });
            }
            EquivocMirOperationKind::ReadImagePixel { image, x, y } => {
                block_builder.add_instruction(EquivocLirInstruction::ReadImagePixel {
                    out: to_lir_var(first_result(op), type_map),
                    image: to_lir_var(*image, type_map),
                    x: to_lir_var(*x, type_map),
                    y: to_lir_var(*y, type_map),
                });
            }
            EquivocMirOperationKind::WriteImagePixel { image, x, y, pixel } => {
                block_builder.add_instruction(EquivocLirInstruction::WriteImagePixel {
                    image: to_lir_var(*image, type_map),
                    x: to_lir_var(*x, type_map),
                    y: to_lir_var(*y, type_map),
                    pixel: to_lir_var(*pixel, type_map),
                });
            }
            EquivocMirOperationKind::LoadImage { path } => {
                let out = to_lir_var(first_result(op), type_map);
                refresh_block!(next_id => EquivocLirTerminateInstruction::LoadImage {
                    out,
                    path: to_lir_var(*path, type_map),
                    next: next_id,
                });
                refresh_block!(next_id => EquivocLirTerminateInstruction::WaitForLoadImage {
                    image: out,
                    next: next_id,
                });
            }
            EquivocMirOperationKind::CallFunction { name, args } => {
                refresh_block!(next_id => EquivocLirTerminateInstruction::CallFunction {
                    out: op.results.first().map(|out| to_lir_var(*out, type_map)),
                    name: name.clone(),
                    args: args.iter().map(|arg| to_lir_var(*arg, type_map)).collect(),
                    next: next_id,
                });
            }
            EquivocMirOperationKind::Return { value } => {
                let block = block_builder.finish(EquivocLirTerminateInstruction::Return {
                    value: value.map(|value| to_lir_var(value, type_map)),
                });
                lir_builder.add_basic_block(block);
                return;
            }
            EquivocMirOperationKind::If {
                condition,
                then_region,
                else_region,
            } => {
                let then_block = lir_builder.next_block();
                let else_block = lir_builder.next_block();
                refresh_block!(_next_block => EquivocLirTerminateInstruction::If {
                    condition: to_lir_var(*condition, type_map),
                    then_block: then_block.id(),
                    else_block: else_block.id(),
                });
                let next_id = block_builder.id();
                build_blocks_from_region(
                    then_region,
                    type_map,
                    lir_builder,
                    then_block,
                    Box::new(|mut block_builder| {
                        add_region_result_assigns(
                            &mut block_builder,
                            &op.results,
                            &then_region.results,
                            type_map,
                        );
                        block_builder.finish(EquivocLirTerminateInstruction::Next {
                            next_block: next_id,
                        })
                    }),
                );
                build_blocks_from_region(
                    else_region,
                    type_map,
                    lir_builder,
                    else_block,
                    Box::new(|mut block_builder| {
                        add_region_result_assigns(
                            &mut block_builder,
                            &op.results,
                            &else_region.results,
                            type_map,
                        );
                        block_builder.finish(EquivocLirTerminateInstruction::Next {
                            next_block: next_id,
                        })
                    }),
                );
            }
            EquivocMirOperationKind::For {
                count,
                index,
                carried,
                reductions,
                body,
                ..
            } => {
                assert!(
                    reductions.is_empty(),
                    "MIR reductions are represented but not lowered to LIR yet"
                );
                let for_block = lir_builder.next_block();
                refresh_block!(next_block => EquivocLirTerminateInstruction::For {
                    loop_count: to_lir_var(*count, type_map),
                    loop_index: to_lir_var(*index, type_map),
                    loop_block: for_block.id(),
                    next_block,
                });
                build_blocks_from_region(
                    body,
                    type_map,
                    lir_builder,
                    for_block,
                    Box::new(|mut block_builder| {
                        add_loop_carried_assigns(&mut block_builder, carried, type_map);
                        block_builder.finish(EquivocLirTerminateInstruction::Continue)
                    }),
                );
            }
            EquivocMirOperationKind::Loop { .. }
            | EquivocMirOperationKind::Break { .. }
            | EquivocMirOperationKind::Continue { .. } => {
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
        infer_region_types(&func.body, &mut type_map);
    }
    infer_region_types(&mir.main_region, &mut type_map);

    let mut lir_builder = EquivocLirBuilder::new();

    for func in &mir.functions {
        let lir_args = func
            .args
            .iter()
            .map(|value| to_lir_var(*value, &type_map))
            .collect();
        let entry_block = lir_builder.next_block();
        let entry_id = entry_block.id();
        build_blocks_from_region(
            &func.body,
            &type_map,
            &mut lir_builder,
            entry_block,
            Box::new(|block_builder| {
                block_builder.finish(EquivocLirTerminateInstruction::Return { value: None })
            }),
        );
        lir_builder.add_function(EquivocLirFunction {
            name: func.name.clone(),
            args: lir_args,
            entry_point: entry_id,
        });
    }

    let entry_block = lir_builder.next_block();
    let entry_id = entry_block.id();
    build_blocks_from_region(
        &mir.main_region,
        &type_map,
        &mut lir_builder,
        entry_block,
        Box::new(|block_builder| {
            block_builder.finish(EquivocLirTerminateInstruction::Return { value: None })
        }),
    );

    lir_builder.finish(entry_id)
}
