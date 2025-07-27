//! Target-specific code generation implementations
//! 
//! This module contains platform-specific implementations for different target architectures.
//! Each target implements the `Target` trait to provide platform-specific assembly generation.

mod base;
mod windows;
mod linux;
mod macos;
mod macos_arm64;

pub use base::{Target, TargetPlatform, CallingConvention};
pub use windows::WindowsX64Target;
pub use linux::LinuxX64Target;
pub use macos::MacOSX64Target;
pub use macos_arm64::MacOSArm64Target;

/// Factory function to create target instances
pub fn create_target(platform: TargetPlatform) -> Box<dyn Target> {
    match platform {
        TargetPlatform::WindowsX64 => Box::new(WindowsX64Target),
        TargetPlatform::LinuxX64 => Box::new(LinuxX64Target),
        TargetPlatform::MacOSX64 => Box::new(MacOSX64Target),
        TargetPlatform::MacOSArm64 => Box::new(MacOSArm64Target),
    }
}

/// Helper function to parse target platform from string
pub fn parse_target_platform(target_str: &str) -> Result<TargetPlatform, String> {
    match target_str.to_lowercase().as_str() {
        "windows" | "win" | "windows-x64" | "win64" => Ok(TargetPlatform::WindowsX64),
        "linux" | "linux-x64" | "linux64" => Ok(TargetPlatform::LinuxX64),
        "macos" | "darwin" | "macos-x64" | "darwin-x64" => Ok(TargetPlatform::MacOSX64),
        "macos-arm64" | "darwin-arm64" | "macos-aarch64" | "darwin-aarch64" => Ok(TargetPlatform::MacOSArm64),
        "macos-auto" | "darwin-auto" => detect_macos_architecture(),
        _ => Err(format!("Unknown target platform: {}", target_str)),
    }
}

/// Automatically detect macOS architecture
pub fn detect_macos_architecture() -> Result<TargetPlatform, String> {
    #[cfg(target_os = "macos")]
    {
        match std::env::consts::ARCH {
            "aarch64" => Ok(TargetPlatform::MacOSArm64),
            "x86_64" => Ok(TargetPlatform::MacOSX64),
            arch => Err(format!("Unsupported macOS architecture: {}", arch)),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Architecture detection is only available on macOS".to_string())
    }
}