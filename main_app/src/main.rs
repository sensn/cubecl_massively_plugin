use std::sync::Arc;
use crossbeam_channel::{unbounded, Sender};
use cubecl::wgpu::{WgpuDevice, WgpuRuntime};
use massively::{Executor, DeviceVec};

mod plugins; // Modul laden
use plugins::{MonolithicPlugin, get_plugin_registry};

// ... (Der Rest deiner main.rs bleibt exakt identisch!)

pub struct GpuCommand {
    pub plugin: Arc<dyn MonolithicPlugin>,
    pub input_vec: Arc<DeviceVec<WgpuRuntime, u32>>, 
    pub output_vec: Arc<DeviceVec<WgpuRuntime, u32>>,
}

pub struct AsyncGpuQueue {
    sender: Sender<GpuCommand>,
}

impl AsyncGpuQueue {
    pub fn start(exec: Executor<WgpuRuntime>) -> Self {
        let (sender, receiver) = unbounded::<GpuCommand>();

        std::thread::spawn(move || {
            println!("[Queue Worker] Bereit für Inferenz-Befehle...");
            while let Ok(cmd) = receiver.recv() {
                let input_slice = cmd.input_vec.slice(..);
                // Erzeuge ein mutables Slice direkt aus dem langlebigen Arc
                // Da wir im monolithischen Scope sind, erlaubt uns das typsichere Aliasing das problemlose Ausleihen
                let mut output_slice = unsafe {
                    let ptr = cmd.output_vec.as_ref() as *const DeviceVec<WgpuRuntime, u32> as *mut DeviceVec<WgpuRuntime, u32>;
                    (*ptr).slice_mut(..)
                };

                println!("[Queue Worker] Starte Kernel-Ausführung: '{}'", cmd.plugin.name());
                if let Ok(_) = cmd.plugin.execute(&exec, input_slice, &mut output_slice) {
                    // Host-Transfer direkt im Worker-Thread ausführen
                    if let Ok(data) = exec.to_host(&output_slice) {
                        println!("[CALLBACK] GPU Inferenz erfolgreich beendet! Ergebnis-Vektor: {:?}", data);
                    }
                }
            }
        });
        Self { sender }
    }

    pub fn submit(&self, cmd: GpuCommand) {
        self.sender.send(cmd).unwrap();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialisiere massively Executor
    let exec = Executor::<WgpuRuntime>::new(WgpuDevice::DefaultDevice);
    
    // Starte die Queue und übergebe den echten, langlebigen Executor per Value
    let gpu_queue = AsyncGpuQueue::start(exec.clone());
    println!("[Main] Monolithisches System erfolgreich gestartet!");

    // 2. Erzeuge Daten direkt auf der GPU
    let input_gpu = Arc::new(exec.to_device(&[1_u32, 2, 3, 4]));
    let output_gpu = Arc::new(exec.alloc::<u32>(4));

    // 3. Durchsuche die monolithische Registry nach unserem Plugin
    let registry = get_plugin_registry();
    if let Some(target_plugin) = registry.first() {
        let command = GpuCommand {
            plugin: target_plugin.clone(),
            input_vec: input_gpu,
            output_vec: output_gpu,
        };

        println!("[Main Thread] Pushe Job in die monolithische Queue...");
        gpu_queue.submit(command);
    }

    // Zeit für den Hintergrundthread einräumen
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(())
}
