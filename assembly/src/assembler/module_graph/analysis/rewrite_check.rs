use alloc::sync::Arc;
use core::ops::ControlFlow;

use crate::{
    assembler::{
        module_graph::{CallerInfo, NameResolver},
        ModuleIndex, ResolvedTarget,
    },
    ast::{visit::Visit, InvocationTarget, InvokeKind, Module},
    diagnostics::SourceFile,
    AssemblyError, Spanned,
};

/// [MaybeRewriteCheck] is a simple analysis pass over a [Module], that looks for evidence that new
/// information has been found that would result in at least one rewrite to the module body.
///
/// This pass is intended for modules that were already added to a [ModuleGraph], and so have been
/// rewritten at least once before. When new modules are added to the graph, the introduction of those
/// modules may allow us to resolve invocation targets that were previously unresolvable, or that
/// resolved as phantoms due to missing definitions. When that occurs, we want to go back and rewrite
/// all of the modules that can be further refined as a result of that additional information.
pub struct MaybeRewriteCheck<'a, 'b: 'a> {
    resolver: &'a NameResolver<'b>,
}
impl<'a, 'b: 'a> MaybeRewriteCheck<'a, 'b> {
    /// Create a new instance of this analysis with the given [NameResolver].
    pub fn new(resolver: &'a NameResolver<'b>) -> Self {
        Self { resolver }
    }

    /// Run the analysis, returning either a boolean answer, or an error that was found during analysis.
    pub fn check(&self, module_id: ModuleIndex, module: &Module) -> Result<bool, AssemblyError> {
        let mut visitor = RewriteCheckVisitor {
            resolver: self.resolver,
            module_id,
            source_file: module.source_file(),
        };
        match visitor.visit_module(module) {
            ControlFlow::Break(result) => result,
            ControlFlow::Continue(_) => Ok(false),
        }
    }
}

struct RewriteCheckVisitor<'a, 'b: 'a> {
    resolver: &'a NameResolver<'b>,
    module_id: ModuleIndex,
    source_file: Option<Arc<SourceFile>>,
}

impl<'a, 'b: 'a> RewriteCheckVisitor<'a, 'b> {
    fn resolve_target(
        &self,
        kind: InvokeKind,
        target: &InvocationTarget,
    ) -> ControlFlow<Result<bool, AssemblyError>> {
        let caller = CallerInfo {
            span: target.span(),
            source_file: self.source_file.clone(),
            module: self.module_id,
            kind,
        };
        match self.resolver.resolve_target(&caller, target) {
            Err(err) => ControlFlow::Break(Err(err)),
            Ok(ResolvedTarget::Resolved { .. }) => ControlFlow::Break(Ok(true)),
            Ok(ResolvedTarget::Exact { .. } | ResolvedTarget::Phantom(_)) => {
                ControlFlow::Continue(())
            }
            Ok(ResolvedTarget::Cached { .. }) => {
                if let InvocationTarget::MastRoot(_) = target {
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(Ok(true))
                }
            }
        }
    }
}

impl<'a, 'b: 'a> Visit<Result<bool, AssemblyError>> for RewriteCheckVisitor<'a, 'b> {
    fn visit_syscall(
        &mut self,
        target: &InvocationTarget,
    ) -> ControlFlow<Result<bool, AssemblyError>> {
        self.resolve_target(InvokeKind::SysCall, target)
    }
    fn visit_call(
        &mut self,
        target: &InvocationTarget,
    ) -> ControlFlow<Result<bool, AssemblyError>> {
        self.resolve_target(InvokeKind::Call, target)
    }
    fn visit_invoke_target(
        &mut self,
        target: &InvocationTarget,
    ) -> ControlFlow<Result<bool, AssemblyError>> {
        self.resolve_target(InvokeKind::Exec, target)
    }
}
