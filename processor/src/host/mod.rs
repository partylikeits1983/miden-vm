use super::{ExecutionError, Felt, ProcessState};
use crate::MemAdviceProvider;
use vm_core::{crypto::merkle::MerklePath, AdviceInjector, DebugOptions, Word};

pub(super) mod advice;
use advice::{AdviceExtractor, AdviceProvider};

#[cfg(feature = "std")]
mod debug;

// HOST TRAIT
// ================================================================================================

/// Defines an interface by which the VM can make requests to the host.
///
/// There are three variants of requests, these can get advice, set advice and invoke the
/// debug handler. The requests are specified by the [AdviceExtractor], [AdviceInjector] and
/// [DebugOptions] enums which target the `get_advice`, `set_advice` and `on_debug` methods
/// respectively. The host is responsible for handling the requests and returning the results to
/// the VM in the form of [HostResponse]. The host is provided with a reference to the current
/// state of the VM ([ProcessState]), which it can use to extract the data required to fulfill the
/// request.
pub trait Host {
    // REQUIRED METHODS
    // --------------------------------------------------------------------------------------------

    /// Returns the requested advice, specified by [AdviceExtractor], from the host to the VM.
    fn get_advice<S: ProcessState>(
        &mut self,
        process: &S,
        extractor: AdviceExtractor,
    ) -> Result<HostResponse, ExecutionError>;

    /// Sets the requested advice, specified by [AdviceInjector], on the host.
    fn set_advice<S: ProcessState>(
        &mut self,
        process: &S,
        injector: AdviceInjector,
    ) -> Result<HostResponse, ExecutionError>;

    // PROVIDED METHODS
    // --------------------------------------------------------------------------------------------

    /// Creates a "by reference" host for this instance.
    ///
    /// The returned adapter also implements [Host] and will simply mutably borrow this
    /// instance.
    fn by_ref(&mut self) -> &mut Self {
        // this trait follows the same model as
        // [io::Read](https://doc.rust-lang.org/std/io/trait.Read.html#method.by_ref).
        //
        // this approach allows the flexibility to take a host  either as owned or by mutable
        // reference - both equally compatible with the trait requirements as we implement
        // `Host` for mutable references of any type that also implements `Host`.
        self
    }

    /// Handles the event emitted from the VM.
    fn on_event<S: ProcessState>(
        &mut self,
        process: &S,
        event_id: u32,
    ) -> Result<HostResponse, ExecutionError> {
        #[cfg(feature = "std")]
        std::println!(
            "Event with id {} emitted at step {} in context {}",
            event_id,
            process.clk(),
            process.ctx()
        );
        Ok(HostResponse::None)
    }

    /// Handles the debug request from the VM.
    fn on_debug<S: ProcessState>(
        &mut self,
        process: &S,
        options: &DebugOptions,
    ) -> Result<HostResponse, ExecutionError> {
        #[cfg(feature = "std")]
        debug::print_debug_info(process, options);
        Ok(HostResponse::None)
    }

    /// Handles the trace emmited from the VM.
    fn on_trace<S: ProcessState>(
        &mut self,
        process: &S,
        trace_id: u32,
    ) -> Result<HostResponse, ExecutionError> {
        #[cfg(feature = "std")]
        std::println!(
            "Trace with id {} emitted at step {} in context {}",
            trace_id,
            process.clk(),
            process.ctx()
        );
        Ok(HostResponse::None)
    }

    /// Handles the failure of the assertion instruction.
    fn on_assert_failed<S: ProcessState>(&mut self, process: &S, err_code: u32) -> ExecutionError {
        ExecutionError::FailedAssertion {
            clk: process.clk(),
            err_code,
            err_msg: None,
        }
    }

    /// Pops an element from the advice stack and returns it.
    ///
    /// # Errors
    /// Returns an error if the advice stack is empty.
    fn pop_adv_stack<S: ProcessState>(&mut self, process: &S) -> Result<Felt, ExecutionError> {
        let response = self.get_advice(process, AdviceExtractor::PopStack)?;
        Ok(response.into())
    }

    /// Pops a word (4 elements) from the advice stack and returns it.
    ///
    /// Note: a word is popped off the stack element-by-element. For example, a `[d, c, b, a, ...]`
    /// stack (i.e., `d` is at the top of the stack) will yield `[d, c, b, a]`.
    ///
    /// # Errors
    /// Returns an error if the advice stack does not contain a full word.
    fn pop_adv_stack_word<S: ProcessState>(&mut self, process: &S) -> Result<Word, ExecutionError> {
        let response = self.get_advice(process, AdviceExtractor::PopStackWord)?;
        Ok(response.into())
    }

    /// Pops a double word (8 elements) from the advice stack and returns them.
    ///
    /// Note: words are popped off the stack element-by-element. For example, a
    /// `[h, g, f, e, d, c, b, a, ...]` stack (i.e., `h` is at the top of the stack) will yield
    /// two words: `[h, g, f,e ], [d, c, b, a]`.
    ///
    /// # Errors
    /// Returns an error if the advice stack does not contain two words.
    fn pop_adv_stack_dword<S: ProcessState>(
        &mut self,
        process: &S,
    ) -> Result<[Word; 2], ExecutionError> {
        let response = self.get_advice(process, AdviceExtractor::PopStackDWord)?;
        Ok(response.into())
    }

    /// Returns a path to a node at the specified depth and index in a Merkle tree with the
    /// specified root.
    ///
    /// # Errors
    /// Returns an error if:
    /// - A Merkle tree for the specified root cannot be found in this advice provider.
    /// - The specified depth is either zero or greater than the depth of the Merkle tree identified
    ///   by the specified root.
    /// - Path to the node at the specified depth and index is not known to this advice provider.
    fn get_adv_merkle_path<S: ProcessState>(
        &mut self,
        process: &S,
    ) -> Result<MerklePath, ExecutionError> {
        let response = self.get_advice(process, AdviceExtractor::GetMerklePath)?;
        Ok(response.into())
    }
}

impl<'a, H> Host for &'a mut H
where
    H: Host,
{
    fn get_advice<S: ProcessState>(
        &mut self,
        process: &S,
        extractor: AdviceExtractor,
    ) -> Result<HostResponse, ExecutionError> {
        H::get_advice(self, process, extractor)
    }

    fn set_advice<S: ProcessState>(
        &mut self,
        process: &S,
        injector: AdviceInjector,
    ) -> Result<HostResponse, ExecutionError> {
        H::set_advice(self, process, injector)
    }

    fn on_debug<S: ProcessState>(
        &mut self,
        process: &S,
        options: &DebugOptions,
    ) -> Result<HostResponse, ExecutionError> {
        H::on_debug(self, process, options)
    }

    fn on_event<S: ProcessState>(
        &mut self,
        process: &S,
        event_id: u32,
    ) -> Result<HostResponse, ExecutionError> {
        H::on_event(self, process, event_id)
    }

    fn on_trace<S: ProcessState>(
        &mut self,
        process: &S,
        trace_id: u32,
    ) -> Result<HostResponse, ExecutionError> {
        H::on_trace(self, process, trace_id)
    }

    fn on_assert_failed<S: ProcessState>(&mut self, process: &S, err_code: u32) -> ExecutionError {
        H::on_assert_failed(self, process, err_code)
    }
}

// HOST RESPONSE
// ================================================================================================

/// Response returned by the host upon successful execution of a [Host] function.
#[derive(Debug)]
pub enum HostResponse {
    MerklePath(MerklePath),
    DoubleWord([Word; 2]),
    Word(Word),
    Element(Felt),
    None,
}

impl From<HostResponse> for MerklePath {
    fn from(response: HostResponse) -> Self {
        match response {
            HostResponse::MerklePath(path) => path,
            _ => panic!("expected MerklePath, but got {:?}", response),
        }
    }
}

impl From<HostResponse> for Word {
    fn from(response: HostResponse) -> Self {
        match response {
            HostResponse::Word(word) => word,
            _ => panic!("expected Word, but got {:?}", response),
        }
    }
}

impl From<HostResponse> for [Word; 2] {
    fn from(response: HostResponse) -> Self {
        match response {
            HostResponse::DoubleWord(word) => word,
            _ => panic!("expected DoubleWord, but got {:?}", response),
        }
    }
}

impl From<HostResponse> for Felt {
    fn from(response: HostResponse) -> Self {
        match response {
            HostResponse::Element(element) => element,
            _ => panic!("expected Element, but got {:?}", response),
        }
    }
}

// DEFAULT HOST IMPLEMENTATION
// ================================================================================================

/// A default [Host] implementation that provides the essential functionality required by the VM.
pub struct DefaultHost<A> {
    adv_provider: A,
}

impl Default for DefaultHost<MemAdviceProvider> {
    fn default() -> Self {
        Self {
            adv_provider: MemAdviceProvider::default(),
        }
    }
}

impl<A: AdviceProvider> DefaultHost<A> {
    pub fn new(adv_provider: A) -> Self {
        Self { adv_provider }
    }

    #[cfg(any(test, feature = "internals"))]
    pub fn advice_provider(&self) -> &A {
        &self.adv_provider
    }

    #[cfg(any(test, feature = "internals"))]
    pub fn advice_provider_mut(&mut self) -> &mut A {
        &mut self.adv_provider
    }

    pub fn into_inner(self) -> A {
        self.adv_provider
    }
}

impl<A: AdviceProvider> Host for DefaultHost<A> {
    fn get_advice<S: ProcessState>(
        &mut self,
        process: &S,
        extractor: AdviceExtractor,
    ) -> Result<HostResponse, ExecutionError> {
        self.adv_provider.get_advice(process, &extractor)
    }

    fn set_advice<S: ProcessState>(
        &mut self,
        process: &S,
        injector: AdviceInjector,
    ) -> Result<HostResponse, ExecutionError> {
        self.adv_provider.set_advice(process, &injector)
    }
}
