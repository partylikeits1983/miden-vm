use crate::{
    ast::*,
    diagnostics::{Diagnostic, Severity, SourceFile},
    Felt, Span, Spanned,
};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    vec::Vec,
};

use super::{SemanticAnalysisError, SyntaxError};

/// This maintains the state for semantic analysis of a single [Module].
pub struct AnalysisContext {
    source_code: Arc<SourceFile>,
    /// A map of constants to the value of that constant
    constants: BTreeMap<Ident, Constant>,
    procedures: BTreeSet<ProcedureName>,
    errors: Vec<SemanticAnalysisError>,
    warnings_as_errors: bool,
}

impl AnalysisContext {
    pub fn new(source_code: Arc<SourceFile>) -> Self {
        Self {
            source_code,
            constants: Default::default(),
            procedures: Default::default(),
            errors: Default::default(),
            warnings_as_errors: false,
        }
    }

    pub fn set_warnings_as_errors(&mut self, yes: bool) {
        self.warnings_as_errors = yes;
    }

    #[inline(always)]
    pub fn warnings_as_errors(&self) -> bool {
        self.warnings_as_errors
    }

    pub fn source_file(&self) -> Arc<SourceFile> {
        self.source_code.clone()
    }

    pub fn register_procedure_name(&mut self, name: ProcedureName) {
        self.procedures.insert(name);
    }

    /// Define a new constant `name`, bound to `value`
    ///
    /// Returns `Err` if the symbol is already defined
    pub fn define_constant(&mut self, mut constant: Constant) -> Result<(), SyntaxError> {
        // Handle symbol conflicts before eval to make sure we can catch self-referential
        // expressions.
        if let Some(value) = self.constants.get(&constant.name) {
            self.errors.push(SemanticAnalysisError::SymbolConflict {
                span: constant.span(),
                prev_span: value.span(),
            });
            return Ok(());
        }

        match self.const_eval(&constant.value) {
            Ok(value) => {
                constant.value = ConstantExpr::Literal(Span::new(constant.span(), value));
                self.constants.insert(constant.name.clone(), constant);
                Ok(())
            }
            Err(err) => {
                self.errors.push(err);
                let errors = core::mem::take(&mut self.errors);
                Err(SyntaxError {
                    input: self.source_code.clone(),
                    errors,
                })
            }
        }
    }

    fn const_eval(&self, value: &ConstantExpr) -> Result<Felt, SemanticAnalysisError> {
        match value {
            ConstantExpr::Literal(value) => Ok(value.into_inner()),
            ConstantExpr::Var(ref name) => self.get_constant(name),
            ConstantExpr::BinaryOp {
                op,
                ref lhs,
                ref rhs,
                ..
            } => {
                let rhs = self.const_eval(rhs)?;
                let lhs = self.const_eval(lhs)?;
                match op {
                    ConstantOp::Add => Ok(lhs + rhs),
                    ConstantOp::Sub => Ok(lhs - rhs),
                    ConstantOp::Mul => Ok(lhs * rhs),
                    ConstantOp::Div => Ok(lhs / rhs),
                    ConstantOp::IntDiv => Ok(Felt::new(lhs.as_int() / rhs.as_int())),
                }
            }
        }
    }

    /// Get the constant value bound to `name`
    ///
    /// Returns `Err` if the symbol is undefined
    pub fn get_constant(&self, name: &Ident) -> Result<Felt, SemanticAnalysisError> {
        let span = name.span();
        if let Some(expr) = self.constants.get(name) {
            Ok(expr.value.expect_literal())
        } else {
            Err(SemanticAnalysisError::SymbolUndefined { span })
        }
    }

    pub fn error(&mut self, diagnostic: SemanticAnalysisError) {
        self.errors.push(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        if self.warnings_as_errors() {
            return !self.errors.is_empty();
        }
        self.errors
            .iter()
            .any(|err| matches!(err.severity().unwrap_or(Severity::Error), Severity::Error))
    }

    pub fn has_failed(&mut self) -> Result<(), SyntaxError> {
        if self.has_errors() {
            Err(SyntaxError {
                input: self.source_file(),
                errors: core::mem::take(&mut self.errors),
            })
        } else {
            Ok(())
        }
    }

    pub fn into_result(self) -> Result<(), SyntaxError> {
        if self.has_errors() {
            Err(SyntaxError {
                input: self.source_code,
                errors: self.errors,
            })
        } else {
            self.emit_warnings();
            Ok(())
        }
    }

    #[cfg(feature = "std")]
    fn emit_warnings(self) {
        use crate::diagnostics::Report;

        if !self.errors.is_empty() {
            // Emit warnings to stderr
            let warning = Report::from(super::errors::SyntaxWarning {
                input: self.source_code,
                errors: self.errors,
            });
            std::eprintln!("{}", warning);
        }
    }

    #[cfg(not(feature = "std"))]
    fn emit_warnings(self) {}
}
