use super::{mem_ops::local_to_absolute_addr, push_felt, AssemblyContext, SpanBuilder};
use crate::{AssemblyError, Felt, Spanned};
use vm_core::Operation::*;

// CONSTANT INPUTS
// ================================================================================================

/// Appends `PUSH` operation to the span block to push provided constant value onto the stack.
///
/// In cases when the immediate value is 0, `PUSH` operation is replaced with `PAD`. Also, in cases
/// when immediate value is 1, `PUSH` operation is replaced with `PAD INCR` because in most cases
/// this will be more efficient than doing a `PUSH`.
pub fn push_one<T>(imm: T, span: &mut SpanBuilder)
where
    T: Into<Felt>,
{
    push_felt(span, imm.into());
}

/// Appends `PUSH` operations to the span block to push two or more provided constant values onto
/// the stack, up to a maximum of 16 values.
///
/// In cases when the immediate value is 0, `PUSH` operation is replaced with `PAD`. Also, in cases
/// when immediate value is 1, `PUSH` operation is replaced with `PAD INCR` because in most cases
/// this will be more efficient than doing a `PUSH`.
pub fn push_many<T>(imms: &[T], span: &mut SpanBuilder)
where
    T: Into<Felt> + Copy,
{
    imms.iter().for_each(|imm| push_felt(span, (*imm).into()));
}

// ENVIRONMENT INPUTS
// ================================================================================================

/// Appends a sequence of operations to the span needed for executing locaddr.i instruction. This
/// consists of putting i onto the stack and then executing LOCADDR operation.
///
/// # Errors
/// Returns an error if index is greater than the number of procedure locals.
pub fn locaddr(
    span: &mut SpanBuilder,
    index: u16,
    context: &AssemblyContext,
) -> Result<(), AssemblyError> {
    local_to_absolute_addr(span, index, context.unwrap_current_procedure().num_locals())
}

/// Appends CALLER operation to the span which puts the hash of the function which initiated the
/// current SYSCALL onto the stack.
///
/// # Errors
/// Returns an error if the instruction is being executed outside of kernel context.
pub fn caller(span: &mut SpanBuilder, context: &AssemblyContext) -> Result<(), AssemblyError> {
    let current_procedure = context.unwrap_current_procedure();
    if !current_procedure.is_kernel() {
        return Err(AssemblyError::CallerOutsideOfKernel {
            span: current_procedure.span(),
            source_file: current_procedure.source_file(),
        });
    }
    span.add_op(Caller);
    Ok(())
}
