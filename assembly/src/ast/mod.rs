//! Abstract syntax tree (AST) components of Miden programs, modules, and procedures.

mod attribute;
mod block;
mod constants;
mod docstring;
mod form;
mod ident;
mod immediate;
mod imports;
mod instruction;
mod invocation_target;
mod module;
mod op;
mod procedure;
#[cfg(test)]
mod tests;
pub mod visit;

pub use self::{
    attribute::{
        Attribute, AttributeSet, AttributeSetEntry, BorrowedMeta, Meta, MetaExpr, MetaItem,
        MetaKeyValue, MetaList,
    },
    block::Block,
    constants::{Constant, ConstantExpr, ConstantOp},
    docstring::DocString,
    form::Form,
    ident::{CaseKindError, Ident, IdentError},
    immediate::{ErrorCode, ImmFelt, ImmU8, ImmU16, ImmU32, Immediate},
    imports::Import,
    instruction::{DebugOptions, Instruction, SystemEventNode, advice::SignatureKind},
    invocation_target::{InvocationTarget, Invoke, InvokeKind},
    module::{Module, ModuleKind},
    op::Op,
    procedure::*,
    visit::{Visit, VisitMut},
};

pub(crate) type SmallOpsVec = smallvec::SmallVec<[Op; 1]>;

/// Maximum stack index at which a full word can start.
pub const MAX_STACK_WORD_OFFSET: u8 = 12;
