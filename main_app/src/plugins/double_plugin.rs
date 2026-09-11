use cubecl::prelude::*;
use cubecl::wgpu::WgpuRuntime;
use massively::{Executor, DeviceSlice, DeviceSliceMut, vector::map, op::UnaryOp, Error};
use super::MonolithicPlugin;

pub struct Double;

#[cubecl::cube]
impl UnaryOp<u32> for Double {
    type Output = u32;
    fn apply(value: u32) -> u32 {
        value * 2
    }
}

pub struct CubeKDoublePlugin;

impl MonolithicPlugin for CubeKDoublePlugin {
    fn name(&self) -> &str {
        "Monolithisch integrierte CubeK-Pipeline"
    }

    fn execute(
        &self, 
        exec: &Executor<WgpuRuntime>, 
        input: DeviceSlice<WgpuRuntime, u32>, 
        output: &mut DeviceSliceMut<WgpuRuntime, u32>
    ) -> Result<(), Error> {
        // 1. Berechne das verdoppelte Zwischenergebnis (MVec/DeviceVec) auf der GPU
        let result_mvec = map(exec, input, Double)?;
        
        // FIX: Übergib das Ausgabe-Slice mittels .clone() per Wert (by value)
        // anstatt als Referenz (&mut). Das erfüllt alle internen Trait-Bedingungen.
        massively::vector::copy(exec, result_mvec.slice(..), output.clone())?;
        
        Ok(())
    }
}
