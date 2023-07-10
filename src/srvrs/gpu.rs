use nvml_wrapper::{Nvml,error::NvmlError};
use users::get_user_by_uid;
use sysinfo::{Pid, ProcessExt, System, SystemExt, Signal};
use lazy_static::lazy_static;
use anyhow::{anyhow, Error};
use std::{thread::sleep, time};
use log::{debug, info, warn, error, LevelFilter};
use itertools::Itertools; // Dependencies are like microplastics. I love microplastics.

/*
* TODO: Implement this
use lazy_static::lazy_static;
use std::sync::{RwLock, Mutex};

#[derive(Default)]
struct FooBar;
impl FooBar {
    fn foo(&mut self) {}
    fn bar(&self) {}
}

lazy_static! {
    static ref DINGUS: Mutex<FooBar> = Mutex::new(FooBar::default());
    static ref CHOM: RwLock<FooBar> = RwLock::new(FooBar::default());
}

fn main() {
    DINGUS.lock().unwrap().foo();
    DINGUS.lock().unwrap().bar();
    CHOM.write().unwrap().foo();
    CHOM.read().unwrap().bar();
}
* */

lazy_static! {
    static ref NVML: Nvml = Nvml::init().unwrap();

    static ref SYSTEM: System = {
        let mut system = System::new_all(); 
        system.refresh_users_list();
        system
    };
}

#[derive(Debug)]
pub struct GPUprocess {
    name: String,
    pid: usize,
    start_time: u64, 
    device_number: usize,
    uid: usize,
    user: String
}

// Query NVML for the processes running on all GPUs and build a list of them
fn get_processes(nvml: &Nvml, system: &System) -> Result<Vec<GPUprocess>, NvmlError>{
    let nvml_device_count = nvml.device_count()?;
    //system.refresh_users_list();
    let mut gpu_processes = vec![];
    for device_number in 0..nvml_device_count {
        let device = nvml.device_by_index(device_number)?;
        let nvml_processes = device.running_compute_processes_v2()?;
        for proc in nvml_processes {
            if let Some(process) = system.process(Pid::from(proc.pid as usize)) {
                let mut gpu_process = GPUprocess {
                    name: process.name().to_string(),
                    pid: proc.pid as usize,
                    start_time: process.start_time(),
                    device_number: device_number as usize,
                    uid: 0,
                    user: "Unknown".to_string()
                };

                // Sometimes, it's not a sure bet that a UID will be found. So we have to handle
                // that.
                if let Some(user_id) = process.user_id() {
                    let user = get_user_by_uid(**user_id).unwrap();

                    gpu_process.uid = **user_id as usize;
                    gpu_process.user = user.name().to_string_lossy().to_string();
                }
                gpu_processes.push(gpu_process); 
            }
        }
    }
    Ok(gpu_processes)
}

pub fn wait_for_gpu(requesting: usize) -> Result<String, Error> {
    let mut timeout_sec = 3600;
    let wait_sec = 2;
    // Get device count
    // Create a vector of free GPUs
    // Get processes running on GPUs
    // Iterate through the processes, checking what GPU it is running on
    // Remove that GPU from the list
    // If the length of the remaining free GPU list is as long or longer
    // than requesting, then return the first two GPUs.
    // Else, wait a bit and do it again. (Or maybe just shit and die and tell the user to try
    // again later.
    while timeout_sec > 0 {
        let nvml_device_count = (*NVML).device_count()?;
        let gpu_processes = get_processes(&(*NVML), &(*SYSTEM))?;
        // Start with a list of every GPU on the system
        let mut free_gpus: Vec<usize> = (0usize..(nvml_device_count as usize)).collect::<Vec<_>>();
        // Remove GPUs if they are in use
        debug!("Checking for processes...");
        for process in gpu_processes.iter() {
            debug!("{:?}", process);
            //free_gpus.remove(free_gpus.iter().position(|x| *x == process.device_number);
            free_gpus.retain(|x| *x != process.device_number);
        }
        // If we have enough free GPUs, return the first two
        // (this does not account for the GPUs' capabilities.
        if free_gpus.len() >= requesting {
            let gpu_string: String = free_gpus[0..requesting].iter().format(",").to_string();

            info!("GPU(s) found! ({})", gpu_string);
            return Ok(gpu_string);
        }
        info!("Not enough GPUs available. Waiting... ({}s left)", timeout_sec);
        sleep(time::Duration::from_secs(wait_sec));
        timeout_sec -= wait_sec;
    }
    // TODO: How do we tell the user this?
    Err(anyhow!("Could not reserve a GPU in time. Please try again later."))
}


