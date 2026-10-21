pub mod address_book;
pub mod capture;
pub mod crypto;
pub mod discovery;
pub mod file_transfer;
pub mod input;
pub mod server;
pub mod state;
pub mod rendezvous;
pub mod audio;
pub mod policy;
pub mod audit;
pub mod service;
pub mod gui;
pub mod native_qs;
pub mod native_admin;

/// Unconditionally terminates the process and all background worker threads immediately.
pub fn force_terminate_process() -> ! {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::System::Threading::{GetCurrentProcess, TerminateProcess};
        let _ = TerminateProcess(GetCurrentProcess(), 0);
    }
    std::process::exit(0);
}
