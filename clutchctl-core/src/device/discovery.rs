//! Device discovery functionality

use crate::device::{IkkegolDevice, PCsensorDevice, PedalDevice};
use crate::error::Result;
use crate::SUPPORTED_DEVICES;
use log::{debug, info};
use std::sync::Arc;

/// Discover all connected pedal devices
pub fn discover_devices() -> Result<Vec<Arc<dyn crate::device::PedalDevice + Send + Sync>>> {
    let mut devices: Vec<Arc<dyn crate::device::PedalDevice + Send + Sync>> = Vec::new();
    let mut device_id = 0;

    // Get all USB devices
    for device in rusb::devices()?.iter() {
        if let Ok(desc) = device.device_descriptor() {
            // Check if this is a supported device
            for &(vendor_id, product_id, device_type) in SUPPORTED_DEVICES {
                if desc.vendor_id() == vendor_id && desc.product_id() == product_id {
                    debug!("Found {} device at bus {} device {}",
                           device_type, device.bus_number(), device.address());

                    // Route to correct device implementation based on USB ID
                    let device_result: Result<Arc<dyn PedalDevice + Send + Sync>> =
                        match (vendor_id, product_id) {
                            // PCsensor devices use HID protocol
                            (0x3553, 0xb001) | (0x0c45, 0x7403) | (0x0c45, 0x7404) |
                            (0x413d, 0x2107) | (0x5131, 0x2019) => {
                                PCsensorDevice::new(vendor_id, product_id, device_id)
                                    .map(|d| Arc::new(d) as Arc<dyn PedalDevice + Send + Sync>)
                            },
                            // iKKEGOL devices use raw USB protocol
                            (0x1a86, 0xe026) => {
                                IkkegolDevice::new(device, device_id)
                                    .map(|d| Arc::new(d) as Arc<dyn PedalDevice + Send + Sync>)
                            },
                            // Scythe devices would need their own implementation
                            (0x0426, 0x3011) | (0x055a, 0x0998) => {
                                // For now, try the iKKEGOL protocol
                                IkkegolDevice::new(device, device_id)
                                    .map(|d| Arc::new(d) as Arc<dyn PedalDevice + Send + Sync>)
                            },
                            _ => {
                                IkkegolDevice::new(device, device_id)
                                    .map(|d| Arc::new(d) as Arc<dyn PedalDevice + Send + Sync>)
                            }
                        };

                    match device_result {
                        Ok(pedal_device) => {
                            info!("Discovered {} device (ID: {})",
                                  pedal_device.model(), device_id);
                            devices.push(pedal_device);
                            device_id += 1;
                        }
                        Err(e) => {
                            debug!("Failed to initialize device: {}", e);
                        }
                    }
                    break; // Found a match, no need to check other device types
                }
            }
        }
    }

    Ok(devices)
}

/// Find a specific device by ID
pub fn find_device_by_id(id: usize) -> Result<Option<Arc<dyn crate::device::PedalDevice + Send + Sync>>> {
    let devices = discover_devices()?;
    Ok(devices.into_iter().find(|d| d.id() == id))
}