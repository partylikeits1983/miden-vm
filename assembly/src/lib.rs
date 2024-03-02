#![no_std]
#![cfg_attr(feature = "nightly", feature(error_in_core))]

#[macro_use]
extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate std;

use vm_core::{
    code_blocks::CodeBlock,
    crypto,
    errors::KernelError,
    utils::{
        ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable, SliceReader,
    },
    CodeBlockTable, Felt, Kernel, Operation, Program, StarkField, ONE, ZERO,
};

#[cfg(feature = "formatter")]
pub use vm_core::{prettier, utils::DisplayHex};

mod assembler;
pub mod ast;
pub mod diagnostics;
mod errors;
pub mod library;
mod parser;
pub(crate) mod sema;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
#[cfg(test)]
mod tests;

pub use self::assembler::{Assembler, AssemblyContext};
pub use self::ast::{Module, ProcedureName};
pub use self::errors::AssemblyError;
pub use self::library::{
    Library, LibraryError, LibraryNamespace, LibraryPath, MaslLibrary, PathError, Version,
};
pub use self::parser::{SourceLocation, SourceSpan, Span, Spanned};

/// Re-exported for downstream crates
pub use vm_core::utils;

// CONSTANTS
// ================================================================================================

/// The maximum number of constant inputs allowed for the `push` instruction.
const MAX_PUSH_INPUTS: usize = 16;

/// The maximum number of elements that can be popped from the advice stack in a single `adv_push`
/// instruction.
const ADVICE_READ_LIMIT: u8 = 16;

/// The maximum number of bits by which a u32 value can be shifted in a bitwise operation.
const MAX_U32_SHIFT_VALUE: u8 = 31;

/// The maximum number of bits by which a u32 value can be rotated in a bitwise operation.
const MAX_U32_ROTATE_VALUE: u8 = 31;

/// The maximum number of bits allowed for the exponent parameter for exponentiation instructions.
const MAX_EXP_BITS: u8 = 64;

/// The required length of the hexadecimal representation for an input value when more than one hex
/// input is provided to `push` masm operation without period separators.
const HEX_CHUNK_SIZE: usize = 16;
