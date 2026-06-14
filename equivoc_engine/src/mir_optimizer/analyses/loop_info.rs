use crate::mir::{EquivocMir, EquivocMirLoopId, EquivocMirOperationKind, EquivocMirRegion};
use crate::mir_optimizer::{Analyses, MirAnalysis, MirDiagnostic};

#[derive(Debug, Default)]
pub struct LoopInfo {
    pub loops: Vec<EquivocMirLoopId>,
}

pub struct LoopAnalysis;

impl MirAnalysis for LoopAnalysis {
    type Output = LoopInfo;

    fn run<Valid>(
        mir: &EquivocMir,
        _analyses: &Analyses<Valid>,
        _diagnostics: &mut Vec<MirDiagnostic>,
    ) -> Self::Output {
        let mut info = LoopInfo::default();
        collect_region_loops(&mir.main_region, &mut info);
        for function in &mir.functions {
            collect_region_loops(&function.body, &mut info);
        }
        info
    }
}

fn collect_region_loops(region: &EquivocMirRegion, info: &mut LoopInfo) {
    for operation in &region.operations {
        match &operation.kind {
            EquivocMirOperationKind::If {
                then_region,
                else_region,
                ..
            } => {
                collect_region_loops(then_region, info);
                collect_region_loops(else_region, info);
            }
            EquivocMirOperationKind::For { loop_id, body, .. }
            | EquivocMirOperationKind::Loop { loop_id, body, .. } => {
                info.loops.push(*loop_id);
                collect_region_loops(body, info);
            }
            _ => {}
        }
    }
}
