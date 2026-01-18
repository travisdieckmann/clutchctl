//! Device abstraction layer

pub mod discovery;
pub mod ikkegol;
pub mod pcsensor;
pub mod traits;

pub use discovery::discover_devices;
pub use ikkegol::IkkegolDevice;
pub use pcsensor::PCsensorDevice;
pub use traits::{PedalDevice, DeviceCapabilities};