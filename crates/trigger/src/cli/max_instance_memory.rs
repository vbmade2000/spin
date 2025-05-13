use spin_core::async_trait;
use spin_factors::RuntimeFactors;
use spin_factors_executor::{ExecutorHooks, FactorsInstanceBuilder};

/// An [`ExecutorHooks`] that sets the maximum memory allocation limit.
pub struct MaxInstanceMemoryHook {
    max_instance_memory: usize,
}

impl MaxInstanceMemoryHook {
    pub fn new(max_instance_memory: usize) -> Self {
        Self {
            max_instance_memory,
        }
    }
}

#[async_trait]
impl<F: RuntimeFactors, U> ExecutorHooks<F, U> for MaxInstanceMemoryHook {
    fn prepare_instance(&self, builder: &mut FactorsInstanceBuilder<F, U>) -> anyhow::Result<()> {
        builder
            .store_builder()
            .max_memory_size(self.max_instance_memory);
        Ok(())
    }
}
