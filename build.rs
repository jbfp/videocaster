fn main() {
    #[cfg(target_os = "windows")]
    windows::build!(windows::win32::system_services::{SetThreadExecutionState});
}
