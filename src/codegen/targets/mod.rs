//! Target-specific implementations
//! 
//! This module contains concrete implementations of the Target trait
//! for different platforms and architectures.

mod windows_x64;
mod linux_x64;
mod macos_x64;

pub use windows_x64::WindowsX64Target;
pub use linux_x64::LinuxX64Target;
pub use macos_x64::MacOSX64Target;