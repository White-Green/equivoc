use crate::mir::{
    EquivocMir, EquivocMirOperation, EquivocMirOperationId, EquivocMirOperationKind,
    EquivocMirRegion, EquivocMirValueId,
};
use crate::mir_optimizer::{Analyses, MirAnalysis, MirDiagnostic};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct DefUseInfo {
    pub uses: HashMap<EquivocMirValueId, Vec<EquivocMirOperationId>>,
}

pub struct DefUseAnalysis;

impl MirAnalysis for DefUseAnalysis {
    type Output = DefUseInfo;

    fn run<Valid>(
        mir: &EquivocMir,
        _analyses: &Analyses<Valid>,
        _diagnostics: &mut Vec<MirDiagnostic>,
    ) -> Self::Output {
        let mut info = DefUseInfo::default();
        collect_region_uses(&mir.main_region, &mut info);
        for function in &mir.functions {
            collect_region_uses(&function.body, &mut info);
        }
        info
    }
}

fn collect_region_uses(region: &EquivocMirRegion, info: &mut DefUseInfo) {
    for operation in &region.operations {
        for operand in operation_operands(operation) {
            info.uses.entry(operand).or_default().push(operation.id);
        }
        match &operation.kind {
            EquivocMirOperationKind::If {
                then_region,
                else_region,
                ..
            } => {
                collect_region_uses(then_region, info);
                collect_region_uses(else_region, info);
            }
            EquivocMirOperationKind::For { body, .. }
            | EquivocMirOperationKind::Loop { body, .. } => collect_region_uses(body, info),
            _ => {}
        }
    }
}

fn operation_operands(operation: &EquivocMirOperation) -> Vec<EquivocMirValueId> {
    match &operation.kind {
        EquivocMirOperationKind::If { condition, .. } => vec![*condition],
        EquivocMirOperationKind::For {
            count,
            index,
            carried,
            reductions,
            ..
        } => {
            let mut operands = vec![*count, *index];
            for carried in carried {
                operands.push(carried.initial);
                operands.push(carried.body_result);
            }
            for reduction in reductions {
                operands.push(reduction.initial);
                operands.push(reduction.accumulator);
                operands.push(reduction.reduced_value);
            }
            operands
        }
        EquivocMirOperationKind::Loop {
            loop_id: _,
            carried,
            body: _,
        } => {
            let mut operands = Vec::new();
            for carried in carried {
                operands.push(carried.initial);
                operands.push(carried.body_result);
            }
            operands
        }
        EquivocMirOperationKind::Return { value } => value.iter().copied().collect(),
        EquivocMirOperationKind::CallFunction { args, .. } => args.clone(),
        EquivocMirOperationKind::Add { lhs, rhs }
        | EquivocMirOperationKind::Sub { lhs, rhs }
        | EquivocMirOperationKind::Mul { lhs, rhs }
        | EquivocMirOperationKind::Div { lhs, rhs }
        | EquivocMirOperationKind::Mod { lhs, rhs }
        | EquivocMirOperationKind::Equals { lhs, rhs }
        | EquivocMirOperationKind::NotEquals { lhs, rhs }
        | EquivocMirOperationKind::LessThan { lhs, rhs }
        | EquivocMirOperationKind::LessThanOrEquals { lhs, rhs }
        | EquivocMirOperationKind::GreaterThan { lhs, rhs }
        | EquivocMirOperationKind::GreaterThanOrEquals { lhs, rhs } => vec![*lhs, *rhs],
        EquivocMirOperationKind::LoadImage { path } => vec![*path],
        EquivocMirOperationKind::WriteImage { image, path } => vec![*image, *path],
        EquivocMirOperationKind::ImageWidth { image }
        | EquivocMirOperationKind::ImageHeight { image } => vec![*image],
        EquivocMirOperationKind::ReadImagePixel { image, x, y } => vec![*image, *x, *y],
        EquivocMirOperationKind::WriteImagePixel { image, x, y, pixel } => {
            vec![*image, *x, *y, *pixel]
        }
        EquivocMirOperationKind::Break { .. }
        | EquivocMirOperationKind::Continue { .. }
        | EquivocMirOperationKind::LoadIntegerConst { .. }
        | EquivocMirOperationKind::LoadFloatConst { .. }
        | EquivocMirOperationKind::LoadStringConst { .. }
        | EquivocMirOperationKind::LoadBooleanConst { .. } => Vec::new(),
    }
}
