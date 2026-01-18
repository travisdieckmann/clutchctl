//! USB HID communication utilities using hidapi
//!
//! This module provides cross-platform HID device access using the hidapi library.
//! On Windows, this uses the native HID driver (no Zadig/WinUSB required).
//! On Linux, this uses hidraw.
//! On macOS, this uses IOKit.

use crate::error::{PedalError, Result};
use hidapi::{HidApi, HidDevice};
use once_cell::sync::OnceCell;
use std::sync::Mutex;

/// Global HidApi instance (thread-safe singleton)
static HID_API: OnceCell<Mutex<HidApi>> = OnceCell::new();

/// Get or initialize the global HidApi instance
pub fn get_hid_api() -> Result<std::sync::MutexGuard<'static, HidApi>> {
    let api = HID_API.get_or_try_init(|| {
        HidApi::new()
            .map(Mutex::new)
            .map_err(PedalError::from)
    })?;

    api.lock().map_err(|_| PedalError::Hid("Failed to lock HID API".to_string()))
}

/// Refresh the device list (call after device connect/disconnect)
pub fn refresh_devices() -> Result<()> {
    let mut api = get_hid_api()?;
    api.refresh_devices()?;
    Ok(())
}

/// Open a HID device by vendor and product ID
pub fn open_device(vendor_id: u16, product_id: u16) -> Result<HidDevice> {
    let api = get_hid_api()?;
    api.open(vendor_id, product_id).map_err(PedalError::from)
}

/// Open a HID device by path (useful when multiple devices with same VID/PID)
pub fn open_device_path(path: &std::ffi::CStr) -> Result<HidDevice> {
    let api = get_hid_api()?;
    api.open_path(path).map_err(PedalError::from)
}

/// Device information from HID enumeration
#[derive(Debug, Clone)]
pub struct HidDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub path: std::ffi::CString,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub interface_number: i32,
}

impl HidDeviceInfo {
    /// Create from hidapi DeviceInfo
    pub fn from_hidapi(info: &hidapi::DeviceInfo) -> Self {
        Self {
            vendor_id: info.vendor_id(),
            product_id: info.product_id(),
            path: info.path().to_owned(),
            serial_number: info.serial_number().map(|s| s.to_string()),
            manufacturer: info.manufacturer_string().map(|s| s.to_string()),
            product: info.product_string().map(|s| s.to_string()),
            interface_number: info.interface_number(),
        }
    }
}

/// List all HID devices matching the given vendor and product IDs
pub fn list_devices(vendor_id: u16, product_id: u16) -> Result<Vec<HidDeviceInfo>> {
    let api = get_hid_api()?;

    let devices: Vec<HidDeviceInfo> = api
        .device_list()
        .filter(|d| d.vendor_id() == vendor_id && d.product_id() == product_id)
        .map(HidDeviceInfo::from_hidapi)
        .collect();

    Ok(devices)
}

/// List all HID devices (for debugging)
pub fn list_all_devices() -> Result<Vec<HidDeviceInfo>> {
    let api = get_hid_api()?;

    let devices: Vec<HidDeviceInfo> = api
        .device_list()
        .map(HidDeviceInfo::from_hidapi)
        .collect();

    Ok(devices)
}
