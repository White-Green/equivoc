use crate::mir::EquivocMir;
use crate::mir_optimizer::{
    Analyses, AnalysisEntry, DefUseAnalysis, EffectAnalysis, HListContains, LoopAnalysis,
    MirDiagnostic, MirTransform, MirTransformResult, Succ, Zero,
};
use std::marker::PhantomData;

pub struct RecordAnalysisSummary<
    DefUseIndex = Succ<Succ<Zero>>,
    LoopIndex = Succ<Zero>,
    EffectIndex = Zero,
> {
    _marker: PhantomData<(DefUseIndex, LoopIndex, EffectIndex)>,
}

impl<DefUseIndex, LoopIndex, EffectIndex>
    RecordAnalysisSummary<DefUseIndex, LoopIndex, EffectIndex>
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<DefUseIndex, LoopIndex, EffectIndex> Default
    for RecordAnalysisSummary<DefUseIndex, LoopIndex, EffectIndex>
{
    fn default() -> Self {
        Self::new()
    }
}

pub fn record_analysis_summary() -> RecordAnalysisSummary {
    RecordAnalysisSummary::new()
}

impl<List, DefUseIndex, LoopIndex, EffectIndex> MirTransform<Analyses<List>>
    for RecordAnalysisSummary<DefUseIndex, LoopIndex, EffectIndex>
where
    List: HListContains<AnalysisEntry<DefUseAnalysis>, DefUseIndex>
        + HListContains<AnalysisEntry<LoopAnalysis>, LoopIndex>
        + HListContains<AnalysisEntry<EffectAnalysis>, EffectIndex>,
{
    type Out = Analyses<List>;

    fn name(&self) -> &'static str {
        "record-analysis-summary"
    }

    fn run(
        &mut self,
        _mir: &mut EquivocMir,
        analyses: Analyses<List>,
        diagnostics: &mut Vec<MirDiagnostic>,
    ) -> (Self::Out, MirTransformResult) {
        let def_use = analyses.get::<DefUseAnalysis, DefUseIndex>();
        let loops = analyses.get::<LoopAnalysis, LoopIndex>();
        let effects = analyses.get::<EffectAnalysis, EffectIndex>();
        diagnostics.push(MirDiagnostic {
            pass_name: "record-analysis-summary",
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
