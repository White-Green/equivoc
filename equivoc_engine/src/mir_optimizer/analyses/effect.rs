use crate::mir::{EquivocMir, EquivocMirOperationKind, EquivocMirRegion};
use crate::mir_optimizer::{Analyses, MirAnalysis, MirDiagnostic};

#[derive(Debug, Default)]
pub struct EffectInfo {
    pub read_count: usize,
    pub write_count: usize,
    pub ordered_effect_count: usize,
    pub control_effect_count: usize,
    pub irreversible_effect_count: usize,
}

pub struct EffectAnalysis;

impl MirAnalysis for EffectAnalysis {
    type Output = EffectInfo;

    fn run<Valid>(
        mir: &EquivocMir,
        _analyses: &Analyses<Valid>,
        _diagnostics: &mut Vec<MirDiagnostic>,
    ) -> Self::Output {
        let mut info = EffectInfo::default();
        collect_region_effects(&mir.main_region, &mut info);
        for function in &mir.functions {
            collect_region_effects(&function.body, &mut info);
        }
        info
    }
}

fn collect_region_effects(region: &EquivocMirRegion, info: &mut EffectInfo) {
    for operation in &region.operations {
        info.read_count += operation.effects.reads.len();
        info.write_count += operation.effects.writes.len();
        if operation.effects.ordered_effect {
            info.ordered_effect_count += 1;
        }
        if operation.effects.control_effect {
            info.control_effect_count += 1;
        }
        if operation.effects.irreversible_effect {
            info.irreversible_effect_count += 1;
        }
        match &operation.kind {
            EquivocMirOperationKind::If {
                then_region,
                else_region,
                ..
            } => {
                collect_region_effects(then_region, info);
                collect_region_effects(else_region, info);
            }
            EquivocMirOperationKind::For { body, .. }
            | EquivocMirOperationKind::Loop { body, .. } => collect_region_effects(body, info),
            _ => {}
        }
    }
}
