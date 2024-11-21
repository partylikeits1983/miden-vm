use std::sync::Arc;

use processor::{
    AdviceExtractor, AdviceProvider, ExecutionError, Host, HostResponse, MastForest,
    MemAdviceProvider, ProcessState,
};
use vm_core::{AdviceInjector, DebugOptions};

mod advice;
mod asmop;
mod events;

// TEST HOST
// ================================================================================================
pub struct TestHost<A> {
    pub adv_provider: A,
    pub event_handler: Vec<u32>,
    pub trace_handler: Vec<u32>,
    pub debug_handler: Vec<String>,
}

impl Default for TestHost<MemAdviceProvider> {
    fn default() -> Self {
        Self {
            adv_provider: MemAdviceProvider::default(),
            event_handler: Vec::new(),
            trace_handler: Vec::new(),
            debug_handler: Vec::new(),
        }
    }
}

impl<A: AdviceProvider> Host for TestHost<A> {
    type AdviceProvider = A;

    fn advice_provider(&self) -> &Self::AdviceProvider {
        &self.adv_provider
    }

    fn advice_provider_mut(&mut self) -> &mut Self::AdviceProvider {
        &mut self.adv_provider
    }

    fn get_advice(
        &mut self,
        process: ProcessState,
        extractor: AdviceExtractor,
    ) -> Result<HostResponse, ExecutionError> {
        self.adv_provider.get_advice(process, &extractor)
    }

    fn set_advice(
        &mut self,
        process: ProcessState,
        injector: AdviceInjector,
    ) -> Result<HostResponse, ExecutionError> {
        self.adv_provider.set_advice(process, &injector)
    }

    fn on_event(
        &mut self,
        _process: ProcessState,
        event_id: u32,
    ) -> Result<HostResponse, ExecutionError> {
        self.event_handler.push(event_id);
        Ok(HostResponse::None)
    }

    fn on_trace(
        &mut self,
        _process: ProcessState,
        trace_id: u32,
    ) -> Result<HostResponse, ExecutionError> {
        self.trace_handler.push(trace_id);
        Ok(HostResponse::None)
    }

    fn on_debug(
        &mut self,
        _process: ProcessState,
        _options: &DebugOptions,
    ) -> Result<HostResponse, ExecutionError> {
        self.debug_handler.push(_options.to_string());
        Ok(HostResponse::None)
    }

    fn get_mast_forest(&self, _node_digest: &prover::Digest) -> Option<Arc<MastForest>> {
        // Empty MAST forest store
        None
    }
}
