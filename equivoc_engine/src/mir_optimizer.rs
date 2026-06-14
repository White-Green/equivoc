mod analyses;
mod transforms;

pub use analyses::{
    DefUseAnalysis, DefUseInfo, EffectAnalysis, EffectInfo, LoopAnalysis, LoopInfo,
};
pub use transforms::{RecomputeEffectSummaries, RecordAnalysisSummary, record_analysis_summary};

use crate::mir::EquivocMir;
use std::marker::PhantomData;

pub struct Nil;
pub struct Cons<Head, Tail>(Head, Tail);

pub struct Zero;
pub struct Succ<N>(PhantomData<N>);

pub trait HListContains<T, I> {
    fn get(&self) -> &T;
}

pub trait HListAdded<T> {
    type Output;
    fn added(self, value: T) -> Self::Output;
}

pub trait HListRemoved<T, I> {
    type Output;
    fn removed(self) -> (Self::Output, T);
}

impl<T, Tail> HListContains<T, Zero> for Cons<T, Tail> {
    fn get(&self) -> &T {
        &self.0
    }
}

impl<A, T, Tail, N> HListContains<T, Succ<N>> for Cons<A, Tail>
where
    Tail: HListContains<T, N>,
{
    fn get(&self) -> &T {
        self.1.get()
    }
}

impl<T, Tail> HListAdded<T> for Tail {
    type Output = Cons<T, Tail>;
    fn added(self, value: T) -> Self::Output {
        Cons(value, self)
    }
}

impl<T, Tail> HListRemoved<T, Zero> for Cons<T, Tail> {
    type Output = Tail;
    fn removed(self) -> (Self::Output, T) {
        let Cons(head, tail) = self;
        (tail, head)
    }
}

impl<A, T, Tail, N> HListRemoved<T, Succ<N>> for Cons<A, Tail>
where
    Tail: HListRemoved<T, N>,
{
    type Output = Cons<A, <Tail as HListRemoved<T, N>>::Output>;
    fn removed(self) -> (Self::Output, T) {
        let Cons(head, tail) = self;
        let (tail, removed) = tail.removed();
        (Cons(head, tail), removed)
    }
}

pub struct AnalysisEntry<A: MirAnalysis>(A::Output);

pub type EmptyAnalyses = Analyses<Nil>;

pub struct Analyses<List> {
    list: List,
}

impl Default for EmptyAnalyses {
    fn default() -> Self {
        Self { list: Nil }
    }
}

impl<List> Analyses<List> {
    fn insert<A>(
        self,
        output: A::Output,
    ) -> Analyses<<List as HListAdded<AnalysisEntry<A>>>::Output>
    where
        A: MirAnalysis,
        List: HListAdded<AnalysisEntry<A>>,
    {
        Analyses {
            list: self.list.added(AnalysisEntry(output)),
        }
    }

    fn clear(self) -> EmptyAnalyses {
        EmptyAnalyses::default()
    }

    pub fn get<A, I>(&self) -> &A::Output
    where
        A: MirAnalysis,
        List: HListContains<AnalysisEntry<A>, I>,
    {
        &<List as HListContains<AnalysisEntry<A>, I>>::get(&self.list).0
    }

    pub fn remove<A, I>(
        self,
    ) -> (
        Analyses<<List as HListRemoved<AnalysisEntry<A>, I>>::Output>,
        A::Output,
    )
    where
        A: MirAnalysis,
        List: HListRemoved<AnalysisEntry<A>, I>,
    {
        let (list, entry) = <List as HListRemoved<AnalysisEntry<A>, I>>::removed(self.list);
        (Analyses { list }, entry.0)
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

impl<List, A> MirPipelineStep<Analyses<List>> for RunAnalysis<A>
where
    A: MirAnalysis,
    List: HListAdded<AnalysisEntry<A>>,
{
    type Out = Analyses<<List as HListAdded<AnalysisEntry<A>>>::Output>;

    fn run(
        self,
        mir: &mut EquivocMir,
        analyses: Analyses<List>,
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
            run_transform(record_analysis_summary()),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{
        EquivocMirEffectSummary, EquivocMirFunction, EquivocMirOperation, EquivocMirOperationId,
        EquivocMirOperationKind, EquivocMirRegion, EquivocMirValueData, EquivocMirValueDef,
        EquivocMirValueId, EquivocMirValueType,
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
                run_transform(record_analysis_summary()),
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
            run_transform(NeedsEffectAnalysis::<Zero>::new()),
        ));
    }

    struct NeedsEffectAnalysis<I> {
        _marker: PhantomData<I>,
    }

    impl<I> NeedsEffectAnalysis<I> {
        fn new() -> Self {
            Self {
                _marker: PhantomData,
            }
        }
    }

    impl<List, I> MirTransform<Analyses<List>> for NeedsEffectAnalysis<I>
    where
        List: HListContains<AnalysisEntry<EffectAnalysis>, I>,
    {
        type Out = Analyses<List>;

        fn name(&self) -> &'static str {
            "needs-effect-analysis"
        }

        fn run(
            &mut self,
            _mir: &mut EquivocMir,
            analyses: Analyses<List>,
            _diagnostics: &mut Vec<MirDiagnostic>,
        ) -> (Self::Out, MirTransformResult) {
            let effects = analyses.get::<EffectAnalysis, I>();
            assert_eq!(effects.read_count, 0);
            (analyses, MirTransformResult { changed: false })
        }
    }

    #[test]
    fn analysis_results_can_be_removed_by_type_index() {
        let analyses = EmptyAnalyses::default()
            .insert::<DefUseAnalysis>(DefUseInfo::default())
            .insert::<EffectAnalysis>(EffectInfo::default());

        let (analyses, effects) = analyses.remove::<EffectAnalysis, Zero>();

        assert_eq!(effects.read_count, 0);
        let def_use = analyses.get::<DefUseAnalysis, Zero>();
        assert!(def_use.uses.is_empty());
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
