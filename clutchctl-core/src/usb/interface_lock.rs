//! RAII wrapper for USB interface management

use crate::error::Result;
use rusb::{DeviceHandle, GlobalContext};

/// RAII wrapper for USB interface claiming/releasing
pub struct UsbInterfaceLock<'a> {
    handle: &'a mut DeviceHandle<GlobalContext>,
    interface: u8,
}

impl<'a> UsbInterfaceLock<'a> {
    /// Claim a USB interface
    pub fn new(handle: &'a mut DeviceHandle<GlobalContext>, interface: u8) -> Result<Self> {
        // Set auto-detach kernel driver
        let _ = handle.set_auto_detach_kernel_driver(true);

        // Claim the interface
        handle.claim_interface(interface)?;

        Ok(Self { handle, interface })
    }

    /// Get the device handle
    pub fn handle(&mut self) -> &mut DeviceHandle<GlobalContext> {
        self.handle
    }
}

impl<'a> Drop for UsbInterfaceLock<'a> {
    fn drop(&mut self) {
        // Release the interface when dropped
        let _ = self.handle.release_interface(self.interface);
    }
}