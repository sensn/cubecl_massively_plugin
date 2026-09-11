// main_app/src/plugins/mod.rs
use std::sync::Arc;
use cubecl::wgpu::WgpuRuntime;
use massively::{Executor, DeviceSlice, DeviceSliceMut, Error};

pub trait MonolithicPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn execute(
        &self, 
        exec: &Executor<WgpuRuntime>, 
        input: DeviceSlice<WgpuRuntime, u32>, 
        output: &mut DeviceSliceMut<WgpuRuntime, u32>
    ) -> Result<(), Error>;
}

// Hier binden wir deine konkreten Plugins als Module ein
pub mod double_plugin;

// Eine einfache Funktion, um alle verfügbaren Module zu sammeln
pub fn get_plugin_registry() -> Vec<Arc<dyn MonolithicPlugin>> {
    vec![
        Arc::new(double_plugin::CubeKDoublePlugin),
        // Weitere Plugins werden einfach hier als Arc hinzugefügt!
    ]
}
