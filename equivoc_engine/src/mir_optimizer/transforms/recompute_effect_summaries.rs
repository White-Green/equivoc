use crate::mir::{
    EquivocMir, EquivocMirEffectSummary, EquivocMirMemoryAccess, EquivocMirMemoryRegion,
    EquivocMirMemoryResource, EquivocMirOperationKind, EquivocMirRegion, EquivocMirValueId,
};
use crate::mir_optimizer::{
    Analyses, EmptyAnalyses, MirDiagnostic, MirTransform, MirTransformResult,
};

pub struct RecomputeEffectSummaries;

impl<Valid> MirTransform<Analyses<Valid>> for RecomputeEffectSummaries {
    type Out = EmptyAnalyses;

    fn name(&self) -> &'static str {
        "recompute-effect-summaries"
    }

    fn run(
        &mut self,
        mir: &mut EquivocMir,
        analyses: Analyses<Valid>,
        _diagnostics: &mut Vec<MirDiagnostic>,
    ) -> (Self::Out, MirTransformResult) {
        let mut changed = false;
        changed |= recompute_region_effects(&mut mir.main_region);
        for function in &mut mir.functions {
            changed |= recompute_region_effects(&mut function.body);
        }
        (analyses.clear(), MirTransformResult { changed })
    }
}

fn recompute_region_effects(region: &mut EquivocMirRegion) -> bool {
    let mut changed = false;
    for operation in &mut region.operations {
        match &mut operation.kind {
            EquivocMirOperationKind::If {
                then_region,
                else_region,
                ..
            } => {
                changed |= recompute_region_effects(then_region);
                changed |= recompute_region_effects(else_region);
            }
            EquivocMirOperationKind::For { body, .. }
            | EquivocMirOperationKind::Loop { body, .. } => {
                changed |= recompute_region_effects(body);
            }
            _ => {}
        }
        let effects = infer_operation_effects(&operation.kind);
        if operation.effects != effects {
            operation.effects = effects;
            changed = true;
        }
    }
    changed
}

fn infer_operation_effects(kind: &EquivocMirOperationKind) -> EquivocMirEffectSummary {
    match kind {
        EquivocMirOperationKind::Break { .. }
        | EquivocMirOperationKind::Continue { .. }
        | EquivocMirOperationKind::Return { .. } => EquivocMirEffectSummary {
            control_effect: true,
            ..Default::default()
        },
        EquivocMirOperationKind::CallFunction { .. } => EquivocMirEffectSummary {
            irreversible_effect: true,
            ..Default::default()
        },
        EquivocMirOperationKind::LoadImage { .. } => EquivocMirEffectSummary {
            reads: vec![external_access()],
            irreversible_effect: true,
            ..Default::default()
        },
        EquivocMirOperationKind::WriteImage { image, .. } => EquivocMirEffectSummary {
            reads: vec![whole_image_access(*image)],
            writes: vec![external_access()],
            irreversible_effect: true,
            ..Default::default()
        },
        EquivocMirOperationKind::ReadImagePixel { image, x, y } => EquivocMirEffectSummary {
            reads: vec![image_pixel_access(*image, *x, *y)],
            ..Default::default()
        },
        EquivocMirOperationKind::WriteImagePixel { image, x, y, .. } => EquivocMirEffectSummary {
            writes: vec![image_pixel_access(*image, *x, *y)],
            ..Default::default()
        },
        _ => EquivocMirEffectSummary::default(),
    }
}

fn whole_image_access(image: EquivocMirValueId) -> EquivocMirMemoryAccess {
    EquivocMirMemoryAccess {
        resource: EquivocMirMemoryResource::Image { image },
        region: EquivocMirMemoryRegion::Whole,
    }
}

fn image_pixel_access(
    image: EquivocMirValueId,
    x: EquivocMirValueId,
    y: EquivocMirValueId,
) -> EquivocMirMemoryAccess {
    EquivocMirMemoryAccess {
        resource: EquivocMirMemoryResource::Image { image },
        region: EquivocMirMemoryRegion::ImagePixel { x, y },
    }
}

fn external_access() -> EquivocMirMemoryAccess {
    EquivocMirMemoryAccess {
        resource: EquivocMirMemoryResource::External,
        region: EquivocMirMemoryRegion::Unknown,
    }
}
