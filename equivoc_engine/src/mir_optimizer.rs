use crate::mir::{
    EquivocMir, EquivocMirEffectSummary, EquivocMirLoopId, EquivocMirMemoryAccess,
    EquivocMirMemoryRegion, EquivocMirMemoryResource, EquivocMirOperation, EquivocMirOperationId,
    EquivocMirOperationKind, EquivocMirRegion, EquivocMirValueId,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct AnalysisNil;
pub struct AnalysisCons<Head, Tail>(PhantomData<(Head, Tail)>);

pub type EmptyAnalyses = Analyses<AnalysisNil>;

pub struct Analyses<Valid> {
    store: HashMap<TypeId, Box<dyn Any>>,
    _valid: PhantomData<Valid>,
}

impl Default for EmptyAnalyses {
    fn default() -> Self {
        Self {
            store: HashMap::new(),
            _valid: PhantomData,
        }
    }
}

impl<Valid> Analyses<Valid> {
    fn insert<A>(mut self, output: A::Output) -> Analyses<AnalysisCons<A, Valid>>
    where
        A: MirAnalysis,
    {
        self.store.insert(TypeId::of::<A>(), Box::new(output));
        Analyses {
            store: self.store,
            _valid: PhantomData,
        }
    }

    fn clear(self) -> EmptyAnalyses {
        EmptyAnalyses::default()
    }

    fn get<A>(&self) -> &A::Output
    where
        A: MirAnalysis,
    {
        self.store
            .get(&TypeId::of::<A>())
            .and_then(|output| output.downcast_ref::<A::Output>())
            .expect("analysis result must be present in the typed analysis set")
    }
}

pub trait MirAnalysis: 'static {
    type Output: 'static;

    fn run<Valid>(
        mir: &EquivocMir,
        analyses: &Analyses<Valid>,
        diagnostics: &mut Vec<MirDiagnostic>,
    ) -> Self::Output;
}

pub trait MirTransform<In> {
    type Out;

    fn name(&self) -> &'static str;

    fn run(
        &mut self,
        mir: &mut EquivocMir,
        analyses: In,
        diagnostics: &mut Vec<MirDiagnostic>,
    ) -> (Self::Out, MirTransformResult);
}

#[derive(Debug, Default)]
pub struct MirTransformResult {
    pub changed: bool,
}

#[derive(Debug)]
pub struct MirDiagnostic {
    pub pass_name: &'static str,
    pub message: String,
}

#[derive(Default)]
pub struct MirOptimizerOptions {
    pub record_pass_diagnostics: bool,
}

#[derive(Default)]
pub struct MirPassContext {
    pub options: MirOptimizerOptions,
    pub diagnostics: Vec<MirDiagnostic>,
}

pub struct RunAnalysis<A> {
    _marker: PhantomData<A>,
}

pub fn run_analysis<A>() -> RunAnalysis<A> {
    RunAnalysis {
        _marker: PhantomData,
    }
}

pub struct RunTransform<T> {
    transform: T,
}

pub fn run_transform<T>(transform: T) -> RunTransform<T> {
    RunTransform { transform }
}

pub trait MirPipelineStep<In> {
    type Out;

    fn run(self, mir: &mut EquivocMir, analyses: In, ctx: &mut MirPassContext) -> Self::Out;
}

impl<Valid, A> MirPipelineStep<Analyses<Valid>> for RunAnalysis<A>
where
    A: MirAnalysis,
{
    type Out = Analyses<AnalysisCons<A, Valid>>;

    fn run(
        self,
        mir: &mut EquivocMir,
        analyses: Analyses<Valid>,
        ctx: &mut MirPassContext,
    ) -> Self::Out {
        let output = A::run(mir, &analyses, &mut ctx.diagnostics);
        analyses.insert::<A>(output)
    }
}

impl<In, T> MirPipelineStep<In> for RunTransform<T>
where
    T: MirTransform<In>,
{
    type Out = T::Out;

    fn run(mut self, mir: &mut EquivocMir, analyses: In, ctx: &mut MirPassContext) -> Self::Out {
        let pass_name = self.transform.name();
        let (analyses, result) = self.transform.run(mir, analyses, &mut ctx.diagnostics);
        if ctx.options.record_pass_diagnostics {
            ctx.diagnostics.push(MirDiagnostic {
                pass_name,
                message: format!("changed={}", result.changed),
            });
        }
        analyses
    }
}

pub trait MirPipeline<In> {
    type Out;

    fn run(self, mir: &mut EquivocMir, analyses: In, ctx: &mut MirPassContext) -> Self::Out;
}

impl<In> MirPipeline<In> for () {
    type Out = In;

    fn run(self, _mir: &mut EquivocMir, analyses: In, _ctx: &mut MirPassContext) -> Self::Out {
        analyses
    }
}

impl<In, Head, Tail> MirPipeline<In> for (Head, Tail)
where
    Head: MirPipelineStep<In>,
    Tail: MirPipeline<Head::Out>,
{
    type Out = Tail::Out;

    fn run(self, mir: &mut EquivocMir, analyses: In, ctx: &mut MirPassContext) -> Self::Out {
        let (head, tail) = self;
        let analyses = head.run(mir, analyses, ctx);
        tail.run(mir, analyses, ctx)
    }
}

#[macro_export]
macro_rules! mir_pipeline {
    () => {
        ()
    };
    ($head:expr $(,)?) => {
        ($head, ())
    };
    ($head:expr, $($tail:expr),+ $(,)?) => {
        ($head, $crate::mir_pipeline!($($tail),+))
    };
}

pub fn run_mir_pipeline<P>(mir: &mut EquivocMir, pipeline: P) -> MirPassContext
where
    P: MirPipeline<EmptyAnalyses>,
{
    let mut ctx = MirPassContext::default();
    pipeline.run(mir, EmptyAnalyses::default(), &mut ctx);
    ctx
}

pub fn optimize_source_mir(mir: &mut EquivocMir) -> MirPassContext {
    run_mir_pipeline(
        mir,
        crate::mir_pipeline!(
            run_transform(RecomputeEffectSummaries),
            run_analysis::<DefUseAnalysis>(),
            run_analysis::<LoopAnalysis>(),
            run_analysis::<EffectAnalysis>(),
            run_transform(RecordAnalysisSummary),
        ),
    )
}

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

pub struct RecordAnalysisSummary;

type SourceSummaryAnalyses = Analyses<
    AnalysisCons<
        EffectAnalysis,
        AnalysisCons<LoopAnalysis, AnalysisCons<DefUseAnalysis, AnalysisNil>>,
    >,
>;

impl MirTransform<SourceSummaryAnalyses> for RecordAnalysisSummary {
    type Out = SourceSummaryAnalyses;

    fn name(&self) -> &'static str {
        "record-analysis-summary"
    }

    fn run(
        &mut self,
        _mir: &mut EquivocMir,
        analyses: SourceSummaryAnalyses,
        diagnostics: &mut Vec<MirDiagnostic>,
    ) -> (Self::Out, MirTransformResult) {
        let def_use = analyses.get::<DefUseAnalysis>();
        let loops = analyses.get::<LoopAnalysis>();
        let effects = analyses.get::<EffectAnalysis>();
        diagnostics.push(MirDiagnostic {
            pass_name: self.name(),
            message: format!(
                "values_with_uses={}, loops={}, reads={}, writes={}",
                def_use.uses.len(),
                loops.loops.len(),
                effects.read_count,
                effects.write_count
            ),
        });
        (analyses, MirTransformResult { changed: false })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{
        EquivocMirFunction, EquivocMirValueData, EquivocMirValueDef, EquivocMirValueType,
    };

    #[test]
    fn pipeline_runs_when_required_analyses_are_explicitly_inserted() {
        let mut mir = sample_mir();
        let ctx = run_mir_pipeline(
            &mut mir,
            crate::mir_pipeline!(
                run_transform(RecomputeEffectSummaries),
                run_analysis::<DefUseAnalysis>(),
                run_analysis::<LoopAnalysis>(),
                run_analysis::<EffectAnalysis>(),
                run_transform(RecordAnalysisSummary),
            ),
        );

        assert_eq!(ctx.diagnostics.len(), 1);
        assert!(ctx.diagnostics[0].message.contains("values_with_uses="));
    }

    #[test]
    fn transform_drops_analysis_until_it_is_inserted_again() {
        fn assert_pipeline_type<P>(_pipeline: P)
        where
            P: MirPipeline<EmptyAnalyses>,
        {
        }

        assert_pipeline_type(crate::mir_pipeline!(
            run_analysis::<EffectAnalysis>(),
            run_transform(RecomputeEffectSummaries),
            run_analysis::<EffectAnalysis>(),
            run_transform(NeedsEffectAnalysis),
        ));
    }

    struct NeedsEffectAnalysis;

    type NeedsEffectInput = Analyses<AnalysisCons<EffectAnalysis, AnalysisNil>>;

    impl MirTransform<NeedsEffectInput> for NeedsEffectAnalysis {
        type Out = NeedsEffectInput;

        fn name(&self) -> &'static str {
            "needs-effect-analysis"
        }

        fn run(
            &mut self,
            _mir: &mut EquivocMir,
            analyses: NeedsEffectInput,
            _diagnostics: &mut Vec<MirDiagnostic>,
        ) -> (Self::Out, MirTransformResult) {
            let effects = analyses.get::<EffectAnalysis>();
            assert_eq!(effects.read_count, 0);
            (analyses, MirTransformResult { changed: false })
        }
    }

    fn sample_mir() -> EquivocMir {
        let result = EquivocMirValueId(0);
        EquivocMir {
            values: vec![EquivocMirValueData {
                ty: EquivocMirValueType::Integer,
                def: EquivocMirValueDef::OperationResult {
                    operation: EquivocMirOperationId(1),
                    result_index: 0,
                },
            }],
            functions: vec![EquivocMirFunction {
                name: "noop".to_string(),
                args: Vec::new(),
                body: EquivocMirRegion {
                    operations: Vec::new(),
                    results: Vec::new(),
                },
            }],
            main_region: EquivocMirRegion {
                operations: vec![EquivocMirOperation {
                    id: EquivocMirOperationId(1),
                    results: vec![result],
                    kind: EquivocMirOperationKind::LoadIntegerConst { value: 1 },
                    effects: EquivocMirEffectSummary::default(),
                }],
                results: Vec::new(),
            },
        }
    }
}
