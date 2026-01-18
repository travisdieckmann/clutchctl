//! Device discovery functionality

use crate::device::{IkkegolDevice, PCsensorDevice, PedalDevice};
use crate::error::Result;
use crate::usb::{get_hid_api, HidDeviceInfo};
use crate::SUPPORTED_DEVICES;
use log::{debug, info};
use std::sync::Arc;

/// Device info collected during enumeration (before opening devices)
struct DiscoveredDeviceInfo {
    vendor_id: u16,
    product_id: u16,
    device_type: &'static str,
    hid_info: HidDeviceInfo,
}

/// Discover all connected pedal devices
pub fn discover_devices() -> Result<Vec<Arc<dyn crate::device::PedalDevice + Send + Sync>>> {
    let mut devices: Vec<Arc<dyn crate::device::PedalDevice + Send + Sync>> = Vec::new();
    let mut device_id = 0;

    // Track which device paths we've already processed (to avoid duplicates from multiple interfaces)
    let mut processed_devices: std::collections::HashSet<(u16, u16, String)> = std::collections::HashSet::new();

    // Collect device info while holding the HID API lock, then release it
    // This avoids deadlock when device constructors try to open devices
    let discovered_devices: Vec<DiscoveredDeviceInfo> = {
        let api = get_hid_api()?;

        let mut found = Vec::new();

        debug!("Enumerating HID devices...");

        // Iterate through all HID devices
        for device_info in api.device_list() {
            let vendor_id = device_info.vendor_id();
            let product_id = device_info.product_id();

            debug!("Checking HID device: VID={:04x} PID={:04x}", vendor_id, product_id);

            // Check if this is a supported device
            for &(supported_vid, supported_pid, device_type) in SUPPORTED_DEVICES {
                if vendor_id == supported_vid && product_id == supported_pid {
                    // Create a unique key for this physical device
                    // Use serial number if available, otherwise use path
                    let device_key = (
                        vendor_id,
                        product_id,
                        device_info.serial_number()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| device_info.path().to_string_lossy().to_string()),
                    );

                    // Skip if we've already processed this device
                    if processed_devices.contains(&device_key) {
                        continue;
                    }

                    debug!("Found {} device: VID={:04x} PID={:04x} interface={}",
                           device_type, vendor_id, product_id, device_info.interface_number());

                    // Collect device info
                    let info = HidDeviceInfo::from_hidapi(device_info);
                    found.push(DiscoveredDeviceInfo {
                        vendor_id,
                        product_id,
                        device_type,
                        hid_info: info,
                    });

                    processed_devices.insert(device_key);
                    break; // Found a match, no need to check other device types
                }
            }
        }

        found
    }; // HID API lock is released here

    // Now open devices without holding the HID API lock
    for discovered in discovered_devices {
        debug!("Opening {} device: VID={:04x} PID={:04x}",
               discovered.device_type, discovered.vendor_id, discovered.product_id);

        let device_result: Result<Arc<dyn PedalDevice + Send + Sync>> =
            match (discovered.vendor_id, discovered.product_id) {
                // PCsensor devices use HID protocol
                (0x3553, 0xb001) | (0x0c45, 0x7403) | (0x0c45, 0x7404) |
                (0x413d, 0x2107) | (0x5131, 0x2019) => {
                    PCsensorDevice::new(discovered.hid_info, device_id)
                        .map(|d| Arc::new(d) as Arc<dyn PedalDevice + Send + Sync>)
                },
                // iKKEGOL devices
                (0x1a86, 0xe026) => {
                    IkkegolDevice::new(discovered.hid_info, device_id)
                        .map(|d| Arc::new(d) as Arc<dyn PedalDevice + Send + Sync>)
                },
                // Scythe devices - try iKKEGOL protocol
                (0x0426, 0x3011) | (0x055a, 0x0998) => {
                    IkkegolDevice::new(discovered.hid_info, device_id)
                        .map(|d| Arc::new(d) as Arc<dyn PedalDevice + Send + Sync>)
                },
                _ => {
                    IkkegolDevice::new(discovered.hid_info, device_id)
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
    }

    Ok(devices)
}

/// Find a specific device by ID
pub fn find_device_by_id(id: usize) -> Result<Option<Arc<dyn crate::device::PedalDevice + Send + Sync>>> {
    let devices = discover_devices()?;
    Ok(devices.into_iter().find(|d| d.id() == id))
}
