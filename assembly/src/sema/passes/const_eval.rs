use crate::{
    ast::*,
    sema::{AnalysisContext, SemanticAnalysisError},
    Felt, Span, Spanned,
};
use core::ops::ControlFlow;

pub struct ConstEvalVisitor<'analyzer> {
    analyzer: &'analyzer mut AnalysisContext,
}
impl<'analyzer> ConstEvalVisitor<'analyzer> {
    pub fn new(analyzer: &'analyzer mut AnalysisContext) -> Self {
        Self { analyzer }
    }
}
impl<'analyzer> ConstEvalVisitor<'analyzer> {
    fn eval_const<T>(&mut self, imm: &mut Immediate<T>) -> ControlFlow<()>
    where
        T: TryFrom<u64>,
    {
        match imm {
            Immediate::Value(_) => ControlFlow::Continue(()),
            Immediate::Constant(ref name) => {
                let span = name.span();
                match self.analyzer.get_constant(name) {
                    Ok(value) => match T::try_from(value.as_int()) {
                        Ok(value) => {
                            *imm = Immediate::Value(Span::new(span, value));
                        }
                        Err(_) => {
                            self.analyzer.error(SemanticAnalysisError::ImmediateOverflow { span });
                        }
                    },
                    Err(error) => {
                        self.analyzer.error(error);
                    }
                }
                ControlFlow::Continue(())
            }
        }
    }
}
impl<'analyzer> VisitMut for ConstEvalVisitor<'analyzer> {
    fn visit_mut_immediate_u8(&mut self, imm: &mut Immediate<u8>) -> ControlFlow<()> {
        self.eval_const(imm)
    }
    fn visit_mut_immediate_u16(&mut self, imm: &mut Immediate<u16>) -> ControlFlow<()> {
        self.eval_const(imm)
    }
    fn visit_mut_immediate_u32(&mut self, imm: &mut Immediate<u32>) -> ControlFlow<()> {
        self.eval_const(imm)
    }
    fn visit_mut_immediate_error_code(&mut self, imm: &mut Immediate<u32>) -> ControlFlow<()> {
        self.eval_const(imm)
    }
    fn visit_mut_immediate_felt(&mut self, imm: &mut Immediate<Felt>) -> ControlFlow<()> {
        match imm {
            Immediate::Value(_) => ControlFlow::Continue(()),
            Immediate::Constant(ref name) => {
                let span = name.span();
                match self.analyzer.get_constant(name) {
                    Ok(value) => {
                        *imm = Immediate::Value(Span::new(span, value));
                    }
                    Err(error) => {
                        self.analyzer.error(error);
                    }
                }
                ControlFlow::Continue(())
            }
        }
    }
}
