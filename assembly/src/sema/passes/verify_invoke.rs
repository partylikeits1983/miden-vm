use alloc::collections::BTreeSet;
use core::ops::ControlFlow;

use crate::{
    ast::*,
    sema::{AnalysisContext, SemanticAnalysisError},
    Span, Spanned,
};

pub struct VerifyInvokeTargets<'a> {
    analyzer: &'a mut AnalysisContext,
    module: &'a mut Module,
    procedures: &'a BTreeSet<ProcedureName>,
    current_procedure: ProcedureName,
    invoked: BTreeSet<Invoke>,
}
impl<'a> VerifyInvokeTargets<'a> {
    pub fn new(
        analyzer: &'a mut AnalysisContext,
        module: &'a mut Module,
        procedures: &'a BTreeSet<ProcedureName>,
        current_procedure: ProcedureName,
    ) -> Self {
        Self {
            analyzer,
            module,
            procedures,
            current_procedure,
            invoked: Default::default(),
        }
    }
}
impl<'a> VerifyInvokeTargets<'a> {
    fn resolve_local(&mut self, name: &ProcedureName) -> ControlFlow<()> {
        if !self.procedures.contains(name) {
            self.analyzer
                .error(SemanticAnalysisError::SymbolUndefined { span: name.span() });
        }
        ControlFlow::Continue(())
    }
    fn resolve_external(
        &mut self,
        name: &ProcedureName,
        module: &Ident,
    ) -> Option<InvocationTarget> {
        match self.module.resolve_import_mut(module) {
            Some(import) => {
                import.uses += 1;
                Some(InvocationTarget::AbsoluteProcedurePath {
                    name: name.clone(),
                    path: import.path.clone(),
                })
            }
            None => {
                self.analyzer.error(SemanticAnalysisError::MissingImport { span: name.span() });
                None
            }
        }
    }
}
impl<'a> VisitMut for VerifyInvokeTargets<'a> {
    fn visit_mut_inst(&mut self, inst: &mut Span<Instruction>) -> ControlFlow<()> {
        let span = inst.span();
        match &**inst {
            Instruction::Caller if self.module.is_kernel() => ControlFlow::Continue(()),
            Instruction::Caller => {
                self.analyzer.error(SemanticAnalysisError::CallerInKernel { span });
                ControlFlow::Continue(())
            }
            _ => visit::visit_mut_inst(self, inst),
        }
    }
    fn visit_mut_procedure_alias(&mut self, alias: &mut ProcedureAlias) -> ControlFlow<()> {
        if let Some(import) = self.module.resolve_import_mut(alias.name().as_ref()) {
            import.uses += 1;
        }
        ControlFlow::Continue(())
    }
    fn visit_mut_procedure(&mut self, procedure: &mut Procedure) -> ControlFlow<()> {
        let result = visit::visit_mut_procedure(self, procedure);
        procedure.extend_invoked(core::mem::take(&mut self.invoked));
        result
    }
    fn visit_mut_syscall(&mut self, target: &mut InvocationTarget) -> ControlFlow<()> {
        if self.module.is_kernel() {
            self.analyzer.error(SemanticAnalysisError::SyscallInKernel {
                span: target.span(),
            });
        }
        match target {
            // Do not analyze syscalls referencing a local name, these
            // will be resolved later in the context of a specific kernel,
            // which may or may not be named `#kernel`, so we can't rewrite
            // this as an absolute path yet, or say for sure if the call is
            // valid
            InvocationTarget::ProcedureName(_) => (),
            _ => self.visit_mut_invoke_target(target)?,
        }
        self.invoked.insert(Invoke::new(InvokeKind::SysCall, target.clone()));
        ControlFlow::Continue(())
    }
    fn visit_mut_call(&mut self, target: &mut InvocationTarget) -> ControlFlow<()> {
        if self.module.is_kernel() {
            self.analyzer.error(SemanticAnalysisError::CallInKernel {
                span: target.span(),
            });
        }
        self.visit_mut_invoke_target(target)?;
        self.invoked.insert(Invoke::new(InvokeKind::Call, target.clone()));
        ControlFlow::Continue(())
    }
    fn visit_mut_exec(&mut self, target: &mut InvocationTarget) -> ControlFlow<()> {
        self.visit_mut_invoke_target(target)?;
        self.invoked.insert(Invoke::new(InvokeKind::Exec, target.clone()));
        ControlFlow::Continue(())
    }
    fn visit_mut_procref(&mut self, target: &mut InvocationTarget) -> ControlFlow<()> {
        self.visit_mut_invoke_target(target)?;
        self.invoked.insert(Invoke::new(InvokeKind::Exec, target.clone()));
        ControlFlow::Continue(())
    }
    fn visit_mut_invoke_target(&mut self, target: &mut InvocationTarget) -> ControlFlow<()> {
        let span = target.span();
        match target {
            InvocationTarget::MastRoot(_) | InvocationTarget::AbsoluteProcedurePath { .. } => (),
            InvocationTarget::ProcedureName(ref name) if name == &self.current_procedure => {
                self.analyzer.error(SemanticAnalysisError::SelfRecursive { span });
            }
            InvocationTarget::ProcedureName(ref name) => {
                return self.resolve_local(name);
            }
            InvocationTarget::ProcedurePath {
                ref name,
                ref module,
            } => {
                if let Some(new_target) = self.resolve_external(name, module) {
                    *target = new_target;
                }
            }
        }
        ControlFlow::Continue(())
    }
}
