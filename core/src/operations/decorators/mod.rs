use alloc::vec::Vec;
use core::fmt;

mod advice;
pub use advice::AdviceInjector;

mod assembly_op;
pub use assembly_op::AssemblyOp;

mod debug;
pub use debug::DebugOptions;

// DECORATORS
// ================================================================================================

/// A set of decorators which can be executed by the VM.
///
/// Executing a decorator does not affect the state of the main VM components such as operand stack
/// and memory. However, decorators may modify the advice provider.
///
/// Executing decorators does not advance the VM clock. As such, many decorators can be executed in
/// a single VM cycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decorator {
    /// Injects new data into the advice provider, as specified by the injector.
    Advice(AdviceInjector),
    /// Adds information about the assembly instruction at a particular index (only applicable in
    /// debug mode).
    AsmOp(AssemblyOp),
    /// Prints out information about the state of the VM based on the specified options. This
    /// decorator is executed only in debug mode.
    Debug(DebugOptions),
    /// Emits an event to the host.
    Event(u32),
    /// Emmits a trace to the host.
    Trace(u32),
}

#[cfg(feature = "formatter")]
impl crate::prettier::PrettyPrint for Decorator {
    fn render(&self) -> crate::prettier::Document {
        crate::prettier::display(self)
    }
}

impl fmt::Display for Decorator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Advice(injector) => write!(f, "advice({injector})"),
            Self::AsmOp(assembly_op) => {
                write!(f, "asmOp({}, {})", assembly_op.op(), assembly_op.num_cycles())
            }
            Self::Debug(options) => write!(f, "debug({options})"),
            Self::Event(event_id) => write!(f, "event({})", event_id),
            Self::Trace(trace_id) => write!(f, "trace({})", trace_id),
        }
    }
}

/// Vector consisting of a tuple of operation index (within a span block) and decorator at that
/// index
pub type DecoratorList = Vec<(usize, Decorator)>;

/// Iterator used to iterate through the decorator list of a span block
/// while executing operation batches of a span block.
pub struct DecoratorIterator<'a> {
    decorators: &'a DecoratorList,
    idx: usize,
}

impl<'a> DecoratorIterator<'a> {
    /// Returns a new instance of decorator iterator instantiated with the provided decorator list.
    pub fn new(decorators: &'a DecoratorList) -> Self {
        Self { decorators, idx: 0 }
    }

    /// Returns the next decorator but only if its position matches the specified position,
    /// otherwise, None is returned.
    #[inline(always)]
    pub fn next_filtered(&mut self, pos: usize) -> Option<&Decorator> {
        if self.idx < self.decorators.len() && self.decorators[self.idx].0 == pos {
            self.idx += 1;
            Some(&self.decorators[self.idx - 1].1)
        } else {
            None
        }
    }
}

impl<'a> Iterator for DecoratorIterator<'a> {
    type Item = &'a Decorator;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.decorators.len() {
            self.idx += 1;
            Some(&self.decorators[self.idx - 1].1)
        } else {
            None
        }
    }
}

// TYPES AND INTERFACES
// ================================================================================================

// Collection of signature schemes supported
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SignatureKind {
    RpoFalcon512,
}

impl fmt::Display for SignatureKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RpoFalcon512 => write!(f, "rpo_falcon512"),
        }
    }
}
