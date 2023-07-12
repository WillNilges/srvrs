use nvml_wrapper::{Nvml,error::NvmlError};
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
}

fn get_free_devices() -> Result<Vec<usize>, NvmlError>{
    let nvml_device_count = (*NVML).device_count()?; // Get every GPU in the system
    let mut free_devices = vec![]; // Assume no free devices
    for device_number in 0..nvml_device_count {
        let device = (*NVML).device_by_index(device_number)?;
        let compute_processes = device.running_compute_processes_v2()?; // Get all processes on current device
        let graphics_processes = device.running_graphics_processes_v2()?; // Get all processes on current device
        // Add to the list of free devices if there are no running processes. 
        if compute_processes.is_empty() && graphics_processes.is_empty() {
            free_devices.push(device_number as usize);
        }
    }
    Ok(free_devices)
}

pub fn wait_for_device(requesting: usize) -> Result<String, Error> {
    let mut timeout_sec = 3600;
    let wait_sec = 2;
    while timeout_sec > 0 {
        let free_devices = get_free_devices()?;
        // If we have enough free GPUs, return the first <requesting> GPUs
        // (this does not account for the GPUs' capabilities.)
        if free_devices.len() >= requesting {
            let device_string: String = free_devices[0..requesting].iter().format(",").to_string();

            info!("GPU(s) found! ({})", device_string);
            return Ok(device_string);
        }
        info!("Not enough GPUs available. Waiting... ({}s left)", timeout_sec);
        sleep(time::Duration::from_secs(wait_sec));
        timeout_sec -= wait_sec;
    }
    // TODO: How do we tell the user this?
    Err(anyhow!("Could not reserve a GPU in time. Please try again later."))
}


