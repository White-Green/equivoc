use crate::frontend_ir::{
    EquivocFrontendFunction, EquivocFrontendInstruction, EquivocFrontendIr,
    EquivocFrontendVariable, FrontendLoopVariableUpdate,
};
use equivoc_engine::mir::{
    EquivocMir, EquivocMirEffectSummary, EquivocMirFunction, EquivocMirImageDependentType,
    EquivocMirImageType, EquivocMirLoopCarried, EquivocMirLoopId, EquivocMirMemoryAccess,
    EquivocMirMemoryRegion, EquivocMirMemoryResource, EquivocMirOperation, EquivocMirOperationId,
    EquivocMirOperationKind, EquivocMirRegion, EquivocMirValueData, EquivocMirValueDef,
    EquivocMirValueId, EquivocMirValueType,
};

pub fn convert_frontend_ir_to_mir(frontend_ir: EquivocFrontendIr) -> EquivocMir {
    let next_synthetic_value_id = max_frontend_value_id(&frontend_ir) + 1;
    let mut converter = FrontendToMirConverter::new(next_synthetic_value_id);
    let functions = frontend_ir
        .functions
        .into_iter()
        .map(|function| converter.convert_function(function))
        .collect();
    let main_region = converter.convert_region(frontend_ir.main_instructions, Vec::new());

    EquivocMir {
        values: converter.values,
        functions,
        main_region,
    }
}

struct FrontendToMirConverter {
    values: Vec<EquivocMirValueData>,
    next_operation_id: u32,
    next_loop_id: u32,
    next_synthetic_value_id: u32,
    loop_stack: Vec<EquivocMirLoopId>,
}

impl FrontendToMirConverter {
    fn new(next_synthetic_value_id: u32) -> Self {
        Self {
            values: Vec::new(),
            next_operation_id: 1,
            next_loop_id: 1,
            next_synthetic_value_id,
            loop_stack: Vec::new(),
        }
    }

    fn convert_function(&mut self, function: EquivocFrontendFunction) -> EquivocMirFunction {
        let args = function
            .args
            .into_iter()
            .map(|arg| {
                let value = self.value_id(arg);
                self.define_value(
                    value,
                    EquivocMirValueType::Unknown,
                    EquivocMirValueDef::FunctionArgument,
                );
                value
            })
            .collect();
        let body = self.convert_region(function.instructions, Vec::new());
        EquivocMirFunction {
            name: function.name,
            args,
            body,
        }
    }

    fn convert_region(
        &mut self,
        instructions: Vec<EquivocFrontendInstruction>,
        results: Vec<EquivocMirValueId>,
    ) -> EquivocMirRegion {
        let operations = instructions
            .into_iter()
            .map(|instruction| self.convert_instruction(instruction))
            .collect();
        EquivocMirRegion {
            operations,
            results,
        }
    }

    fn convert_instruction(
        &mut self,
        instruction: EquivocFrontendInstruction,
    ) -> EquivocMirOperation {
        match instruction {
            EquivocFrontendInstruction::If {
                variables,
                condition,
                then_instructions,
                else_instructions,
            } => {
                let results = variables
                    .iter()
                    .map(|update| self.value_id(update.variable))
                    .collect::<Vec<_>>();
                for result in &results {
                    self.define_value(
                        *result,
                        EquivocMirValueType::Unknown,
                        EquivocMirValueDef::Unknown,
                    );
                }
                let then_results = variables
                    .iter()
                    .map(|update| self.value_id(update.then_variable))
                    .collect();
                let else_results = variables
                    .iter()
                    .map(|update| self.value_id(update.else_variable))
                    .collect();
                let then_region = self.convert_region(then_instructions, then_results);
                let else_region = self.convert_region(else_instructions, else_results);
                let condition = self.value_id(condition);
                self.operation(
                    results,
                    EquivocMirOperationKind::If {
                        condition,
                        then_region,
                        else_region,
                    },
                    EquivocMirEffectSummary::default(),
                )
            }
            EquivocFrontendInstruction::For {
                variable_updates,
                loop_count,
                loop_index,
                instructions,
            } => {
                let loop_id = self.next_loop_id();
                let index = self.value_id(loop_index);
                self.define_value(
                    index,
                    EquivocMirValueType::Integer,
                    EquivocMirValueDef::LoopIndex { loop_id },
                );
                let (results, carried) = self.convert_loop_carried(variable_updates);
                self.loop_stack.push(loop_id);
                let body = self.convert_region(instructions, Vec::new());
                self.loop_stack.pop();
                let count = self.value_id(loop_count);
                self.operation(
                    results,
                    EquivocMirOperationKind::For {
                        loop_id,
                        count,
                        index,
                        carried,
                        reductions: Vec::new(),
                        body,
                    },
                    EquivocMirEffectSummary::default(),
                )
            }
            EquivocFrontendInstruction::Loop {
                variable_updates,
                instructions,
            } => {
                let loop_id = self.next_loop_id();
                let (results, carried) = self.convert_loop_carried(variable_updates);
                self.loop_stack.push(loop_id);
                let body = self.convert_region(instructions, Vec::new());
                self.loop_stack.pop();
                self.operation(
                    results,
                    EquivocMirOperationKind::Loop {
                        loop_id,
                        carried,
                        body,
                    },
                    EquivocMirEffectSummary::default(),
                )
            }
            EquivocFrontendInstruction::Break => self.operation(
                Vec::new(),
                EquivocMirOperationKind::Break {
                    target: self.current_loop_id(),
                },
                EquivocMirEffectSummary {
                    control_effect: true,
                    ..Default::default()
                },
            ),
            EquivocFrontendInstruction::Continue => self.operation(
                Vec::new(),
                EquivocMirOperationKind::Continue {
                    target: self.current_loop_id(),
                },
                EquivocMirEffectSummary {
                    control_effect: true,
                    ..Default::default()
                },
            ),
            EquivocFrontendInstruction::Return { value } => {
                let value = value.map(|value| self.value_id(value));
                self.operation(
                    Vec::new(),
                    EquivocMirOperationKind::Return { value },
                    EquivocMirEffectSummary {
                        control_effect: true,
                        ..Default::default()
                    },
                )
            }
            EquivocFrontendInstruction::CallFunction { out, name, args } => {
                let results = out
                    .map(|out| vec![self.value_id(out)])
                    .unwrap_or_else(Vec::new);
                for result in &results {
                    self.define_value(
                        *result,
                        EquivocMirValueType::Unknown,
                        EquivocMirValueDef::Unknown,
                    );
                }
                let args = args.into_iter().map(|arg| self.value_id(arg)).collect();
                self.operation(
                    results,
                    EquivocMirOperationKind::CallFunction { name, args },
                    EquivocMirEffectSummary {
                        irreversible_effect: true,
                        ..Default::default()
                    },
                )
            }
            EquivocFrontendInstruction::LoadIntegerConst { out, value } => self.result_operation(
                out,
                EquivocMirValueType::Integer,
                EquivocMirOperationKind::LoadIntegerConst { value },
            ),
            EquivocFrontendInstruction::LoadFloatConst { out, value } => self.result_operation(
                out,
                EquivocMirValueType::Float,
                EquivocMirOperationKind::LoadFloatConst { value },
            ),
            EquivocFrontendInstruction::LoadStringConst { out, value } => self.result_operation(
                out,
                EquivocMirValueType::String,
                EquivocMirOperationKind::LoadStringConst { value },
            ),
            EquivocFrontendInstruction::LoadBooleanConst { out, value } => self.result_operation(
                out,
                EquivocMirValueType::Boolean,
                EquivocMirOperationKind::LoadBooleanConst { value },
            ),
            EquivocFrontendInstruction::Add { out, lhs, rhs } => {
                self.binary_result_operation(out, lhs, rhs, |lhs, rhs| {
                    EquivocMirOperationKind::Add { lhs, rhs }
                })
            }
            EquivocFrontendInstruction::Sub { out, lhs, rhs } => {
                self.binary_result_operation(out, lhs, rhs, |lhs, rhs| {
                    EquivocMirOperationKind::Sub { lhs, rhs }
                })
            }
            EquivocFrontendInstruction::Mul { out, lhs, rhs } => {
                self.binary_result_operation(out, lhs, rhs, |lhs, rhs| {
                    EquivocMirOperationKind::Mul { lhs, rhs }
                })
            }
            EquivocFrontendInstruction::Div { out, lhs, rhs } => {
                self.binary_result_operation(out, lhs, rhs, |lhs, rhs| {
                    EquivocMirOperationKind::Div { lhs, rhs }
                })
            }
            EquivocFrontendInstruction::Mod { out, lhs, rhs } => {
                self.binary_result_operation(out, lhs, rhs, |lhs, rhs| {
                    EquivocMirOperationKind::Mod { lhs, rhs }
                })
            }
            EquivocFrontendInstruction::Equals { out, lhs, rhs } => {
                self.compare_result_operation(out, lhs, rhs, |lhs, rhs| {
                    EquivocMirOperationKind::Equals { lhs, rhs }
                })
            }
            EquivocFrontendInstruction::NotEquals { out, lhs, rhs } => self
                .compare_result_operation(out, lhs, rhs, |lhs, rhs| {
                    EquivocMirOperationKind::NotEquals { lhs, rhs }
                }),
            EquivocFrontendInstruction::LessThan { out, lhs, rhs } => self
                .compare_result_operation(out, lhs, rhs, |lhs, rhs| {
                    EquivocMirOperationKind::LessThan { lhs, rhs }
                }),
            EquivocFrontendInstruction::LessThanOrEquals { out, lhs, rhs } => self
                .compare_result_operation(out, lhs, rhs, |lhs, rhs| {
                    EquivocMirOperationKind::LessThanOrEquals { lhs, rhs }
                }),
            EquivocFrontendInstruction::GreaterThan { out, lhs, rhs } => self
                .compare_result_operation(out, lhs, rhs, |lhs, rhs| {
                    EquivocMirOperationKind::GreaterThan { lhs, rhs }
                }),
            EquivocFrontendInstruction::GreaterThanOrEquals { out, lhs, rhs } => self
                .compare_result_operation(out, lhs, rhs, |lhs, rhs| {
                    EquivocMirOperationKind::GreaterThanOrEquals { lhs, rhs }
                }),
            EquivocFrontendInstruction::LoadImage { out, path } => {
                let out = self.value_id(out);
                let path = self.value_id(path);
                self.define_value(
                    out,
                    EquivocMirValueType::Image(EquivocMirImageType { header_source: out }),
                    EquivocMirValueDef::Unknown,
                );
                self.operation(
                    vec![out],
                    EquivocMirOperationKind::LoadImage { path },
                    EquivocMirEffectSummary {
                        reads: vec![Self::external_access()],
                        irreversible_effect: true,
                        ..Default::default()
                    },
                )
            }
            EquivocFrontendInstruction::WriteImage { image, path } => {
                let image = self.value_id(image);
                let path = self.value_id(path);
                self.operation(
                    Vec::new(),
                    EquivocMirOperationKind::WriteImage { image, path },
                    EquivocMirEffectSummary {
                        reads: vec![Self::whole_image_access(image)],
                        writes: vec![Self::external_access()],
                        irreversible_effect: true,
                        ..Default::default()
                    },
                )
            }
            EquivocFrontendInstruction::ImageWidth { out, image } => {
                let image = self.value_id(image);
                self.operation_with_result(
                    out,
                    EquivocMirValueType::Integer,
                    EquivocMirOperationKind::ImageWidth { image },
                    EquivocMirEffectSummary::default(),
                )
            }
            EquivocFrontendInstruction::ImageHeight { out, image } => {
                let image = self.value_id(image);
                self.operation_with_result(
                    out,
                    EquivocMirValueType::Integer,
                    EquivocMirOperationKind::ImageHeight { image },
                    EquivocMirEffectSummary::default(),
                )
            }
            EquivocFrontendInstruction::ReadImagePixel { out, image, x, y } => {
                let out = self.value_id(out);
                let image = self.value_id(image);
                let x = self.value_id(x);
                let y = self.value_id(y);
                self.define_value(
                    out,
                    EquivocMirValueType::Pixel(EquivocMirImageDependentType { image }),
                    EquivocMirValueDef::Unknown,
                );
                self.operation(
                    vec![out],
                    EquivocMirOperationKind::ReadImagePixel { image, x, y },
                    EquivocMirEffectSummary {
                        reads: vec![Self::image_pixel_access(image, x, y)],
                        ..Default::default()
                    },
                )
            }
            EquivocFrontendInstruction::WriteImagePixel { image, x, y, pixel } => {
                let image = self.value_id(image);
                let x = self.value_id(x);
                let y = self.value_id(y);
                let pixel = self.value_id(pixel);
                self.operation(
                    Vec::new(),
                    EquivocMirOperationKind::WriteImagePixel { image, x, y, pixel },
                    EquivocMirEffectSummary {
                        writes: vec![Self::image_pixel_access(image, x, y)],
                        ..Default::default()
                    },
                )
            }
        }
    }

    fn result_operation(
        &mut self,
        out: EquivocFrontendVariable,
        ty: EquivocMirValueType,
        kind: EquivocMirOperationKind,
    ) -> EquivocMirOperation {
        self.operation_with_result(out, ty, kind, EquivocMirEffectSummary::default())
    }

    fn binary_result_operation(
        &mut self,
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
        kind: impl FnOnce(EquivocMirValueId, EquivocMirValueId) -> EquivocMirOperationKind,
    ) -> EquivocMirOperation {
        let lhs = self.value_id(lhs);
        let rhs = self.value_id(rhs);
        self.operation_with_result(
            out,
            EquivocMirValueType::Unknown,
            kind(lhs, rhs),
            EquivocMirEffectSummary::default(),
        )
    }

    fn compare_result_operation(
        &mut self,
        out: EquivocFrontendVariable,
        lhs: EquivocFrontendVariable,
        rhs: EquivocFrontendVariable,
        kind: impl FnOnce(EquivocMirValueId, EquivocMirValueId) -> EquivocMirOperationKind,
    ) -> EquivocMirOperation {
        let lhs = self.value_id(lhs);
        let rhs = self.value_id(rhs);
        self.operation_with_result(
            out,
            EquivocMirValueType::Boolean,
            kind(lhs, rhs),
            EquivocMirEffectSummary::default(),
        )
    }

    fn operation_with_result(
        &mut self,
        out: EquivocFrontendVariable,
        ty: EquivocMirValueType,
        kind: EquivocMirOperationKind,
        effects: EquivocMirEffectSummary,
    ) -> EquivocMirOperation {
        let result = self.value_id(out);
        self.define_value(result, ty, EquivocMirValueDef::Unknown);
        self.operation(vec![result], kind, effects)
    }

    fn operation(
        &mut self,
        results: Vec<EquivocMirValueId>,
        kind: EquivocMirOperationKind,
        effects: EquivocMirEffectSummary,
    ) -> EquivocMirOperation {
        let id = self.next_operation_id();
        self.define_operation_results(id, &results);
        EquivocMirOperation {
            id,
            results,
            kind,
            effects,
        }
    }

    fn define_operation_results(
        &mut self,
        operation: EquivocMirOperationId,
        results: &[EquivocMirValueId],
    ) {
        for (result_index, result) in results.iter().enumerate() {
            self.ensure_value(*result);
            self.values[u32::from(*result) as usize].def = EquivocMirValueDef::OperationResult {
                operation,
                result_index: result_index as u32,
            };
        }
    }

    fn convert_loop_carried(
        &mut self,
        updates: Vec<FrontendLoopVariableUpdate>,
    ) -> (Vec<EquivocMirValueId>, Vec<EquivocMirLoopCarried>) {
        let mut results = Vec::new();
        let mut carried = Vec::new();
        for update in updates {
            let result = self.synthetic_value();
            self.define_value(
                result,
                EquivocMirValueType::Unknown,
                EquivocMirValueDef::Unknown,
            );
            results.push(result);
            carried.push(EquivocMirLoopCarried {
                initial: self.value_id(update.base),
                body_result: self.value_id(update.updated),
                result,
            });
        }
        (results, carried)
    }

    fn next_loop_id(&mut self) -> EquivocMirLoopId {
        let loop_id = EquivocMirLoopId(self.next_loop_id);
        self.next_loop_id += 1;
        loop_id
    }

    fn next_operation_id(&mut self) -> EquivocMirOperationId {
        let operation_id = EquivocMirOperationId(self.next_operation_id);
        self.next_operation_id += 1;
        operation_id
    }

    fn current_loop_id(&self) -> EquivocMirLoopId {
        *self
            .loop_stack
            .last()
            .expect("break or continue must be inside a loop")
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

    fn synthetic_value(&mut self) -> EquivocMirValueId {
        let value = EquivocMirValueId(self.next_synthetic_value_id);
        self.next_synthetic_value_id += 1;
        self.ensure_value(value);
        value
    }

    fn value_id(&mut self, value: EquivocFrontendVariable) -> EquivocMirValueId {
        let value = EquivocMirValueId(u32::from(value));
        self.ensure_value(value);
        value
    }

    fn ensure_value(&mut self, value: EquivocMirValueId) {
        let index = u32::from(value) as usize;
        if self.values.len() <= index {
            self.values
                .resize_with(index + 1, EquivocMirValueData::default);
        }
    }

    fn define_value(
        &mut self,
        value: EquivocMirValueId,
        ty: EquivocMirValueType,
        def: EquivocMirValueDef,
    ) {
        self.ensure_value(value);
        let data = &mut self.values[u32::from(value) as usize];
        data.ty = ty;
        data.def = def;
    }
}

fn max_frontend_value_id(frontend_ir: &EquivocFrontendIr) -> u32 {
    let mut max_id = 0;
    for function in &frontend_ir.functions {
        for arg in &function.args {
            max_id = max_id.max(u32::from(*arg));
        }
        max_id = max_id.max(max_instruction_value_id(&function.instructions));
    }
    max_id.max(max_instruction_value_id(&frontend_ir.main_instructions))
}

fn max_instruction_value_id(instructions: &[EquivocFrontendInstruction]) -> u32 {
    let mut max_id = 0;
    for instruction in instructions {
        match instruction {
            EquivocFrontendInstruction::If {
                variables,
                condition,
                then_instructions,
                else_instructions,
            } => {
                max_id = max_id.max(u32::from(*condition));
                for update in variables {
                    max_id = max_id.max(u32::from(update.variable));
                    max_id = max_id.max(u32::from(update.then_variable));
                    max_id = max_id.max(u32::from(update.else_variable));
                }
                max_id = max_id.max(max_instruction_value_id(then_instructions));
                max_id = max_id.max(max_instruction_value_id(else_instructions));
            }
            EquivocFrontendInstruction::For {
                variable_updates,
                loop_count,
                loop_index,
                instructions,
            } => {
                max_id = max_id.max(u32::from(*loop_count));
                max_id = max_id.max(u32::from(*loop_index));
                max_id = max_id.max(max_loop_update_value_id(variable_updates));
                max_id = max_id.max(max_instruction_value_id(instructions));
            }
            EquivocFrontendInstruction::Loop {
                variable_updates,
                instructions,
            } => {
                max_id = max_id.max(max_loop_update_value_id(variable_updates));
                max_id = max_id.max(max_instruction_value_id(instructions));
            }
            EquivocFrontendInstruction::Break | EquivocFrontendInstruction::Continue => {}
            EquivocFrontendInstruction::Return { value } => {
                if let Some(value) = value {
                    max_id = max_id.max(u32::from(*value));
                }
            }
            EquivocFrontendInstruction::CallFunction { out, args, .. } => {
                if let Some(out) = out {
                    max_id = max_id.max(u32::from(*out));
                }
                for arg in args {
                    max_id = max_id.max(u32::from(*arg));
                }
            }
            EquivocFrontendInstruction::LoadIntegerConst { out, .. }
            | EquivocFrontendInstruction::LoadFloatConst { out, .. }
            | EquivocFrontendInstruction::LoadStringConst { out, .. }
            | EquivocFrontendInstruction::LoadBooleanConst { out, .. }
            | EquivocFrontendInstruction::LoadImage { out, .. }
            | EquivocFrontendInstruction::ImageWidth { out, .. }
            | EquivocFrontendInstruction::ImageHeight { out, .. } => {
                max_id = max_id.max(u32::from(*out));
            }
            EquivocFrontendInstruction::Add { out, lhs, rhs }
            | EquivocFrontendInstruction::Sub { out, lhs, rhs }
            | EquivocFrontendInstruction::Mul { out, lhs, rhs }
            | EquivocFrontendInstruction::Div { out, lhs, rhs }
            | EquivocFrontendInstruction::Mod { out, lhs, rhs }
            | EquivocFrontendInstruction::Equals { out, lhs, rhs }
            | EquivocFrontendInstruction::NotEquals { out, lhs, rhs }
            | EquivocFrontendInstruction::LessThan { out, lhs, rhs }
            | EquivocFrontendInstruction::LessThanOrEquals { out, lhs, rhs }
            | EquivocFrontendInstruction::GreaterThan { out, lhs, rhs }
            | EquivocFrontendInstruction::GreaterThanOrEquals { out, lhs, rhs } => {
                max_id = max_id.max(u32::from(*out));
                max_id = max_id.max(u32::from(*lhs));
                max_id = max_id.max(u32::from(*rhs));
            }
            EquivocFrontendInstruction::WriteImage { image, path } => {
                max_id = max_id.max(u32::from(*image));
                max_id = max_id.max(u32::from(*path));
            }
            EquivocFrontendInstruction::ReadImagePixel { out, image, x, y } => {
                max_id = max_id.max(u32::from(*out));
                max_id = max_id.max(u32::from(*image));
                max_id = max_id.max(u32::from(*x));
                max_id = max_id.max(u32::from(*y));
            }
            EquivocFrontendInstruction::WriteImagePixel { image, x, y, pixel } => {
                max_id = max_id.max(u32::from(*image));
                max_id = max_id.max(u32::from(*x));
                max_id = max_id.max(u32::from(*y));
                max_id = max_id.max(u32::from(*pixel));
            }
        }
    }
    max_id
}

fn max_loop_update_value_id(updates: &[FrontendLoopVariableUpdate]) -> u32 {
    let mut max_id = 0;
    for update in updates {
        max_id = max_id.max(u32::from(update.base));
        max_id = max_id.max(u32::from(update.updated));
    }
    max_id
}
