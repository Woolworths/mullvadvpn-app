use std::borrow::Borrow;
use std::net::IpAddr;
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::ptr;
use std::slice;

use error_chain::ChainedError;
use widestring::WideCString;

use super::system_state::SystemStateWriter;

const DNS_STATE_FILENAME: &'static str = "dns-state-backup";

error_chain!{
    errors{
        /// Failure to initialize WinDns
        Initialization{
            description("Failed to initialize WinDns")
        }

        /// Failure to deinitialize WinDns
        Deinitialization{
            description("Failed to deinitialize WinDns")
        }

        /// Failure to set new DNS servers
        Setting{
            description("Failed to set new DNS servers")
        }

        /// Failure to reset DNS settings
        Resetting{
            description("Failed to reset DNS")
        }

        /// Failure to reset DNS settings from backup
        Recovery{
            description("Failed to recover to backed up system state")
        }
    }
}

pub struct WinDns {
    backup_writer: SystemStateWriter,
}

impl WinDns {
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        unsafe { WinDns_Initialize(Some(error_sink), ptr::null_mut()).into_result()? };

        let backup_writer = SystemStateWriter::new(
            cache_dir
                .as_ref()
                .join(DNS_STATE_FILENAME)
                .into_boxed_path(),
        );
        let mut dns = WinDns { backup_writer };
        if let Err(error) = dns
            .restore_system_backup()
            .chain_err(|| "Failed to restore DNS backup")
        {
            error!("{}", error.display_chain());
        }
        Ok(dns)
    }

    pub fn set_dns(&mut self, servers: &[IpAddr]) -> Result<()> {
        info!(
            "Setting DNS servers - {}",
            servers
                .iter()
                .map(|ip| ip.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
        let widestring_ips = servers
            .iter()
            .map(|ip| ip.to_string().encode_utf16().collect::<Vec<_>>())
            .map(|ip| WideCString::new(ip).unwrap())
            .collect::<Vec<_>>();

        let mut ip_ptrs = widestring_ips
            .iter()
            .map(|ip_cstr| ip_cstr.as_ptr())
            .collect::<Vec<_>>();

        unsafe {
            WinDns_Set(
                ip_ptrs.as_mut_ptr(),
                widestring_ips.len() as u32,
                Some(write_system_state_backup_cb),
                &self.backup_writer as *const _ as *const c_void,
            ).into_result()
        }
    }

    pub fn reset_dns(&mut self) -> Result<()> {
        trace!("Resetting DNS");
        unsafe { WinDns_Reset().into_result()? };

        if let Err(e) = self.backup_writer.remove_backup() {
            warn!("Failed to remove DNS state backup file: {}", e);
        }
        Ok(())
    }

    fn restore_dns_settings(&mut self, data: &[u8]) -> Result<()> {
        unsafe { WinDns_Recover(data.as_ptr(), data.len() as u32) }.into_result()
    }

    fn restore_system_backup(&mut self) -> Result<()> {
        if let Some(previous_state) = self
            .backup_writer
            .read_backup()
            .chain_err(|| "Failed to read backed up DNS state")?
        {
            info!("Restoring DNS state from backup");
            self.restore_dns_settings(&previous_state)
                .chain_err(|| "Failed to restore backed up DNS state")?;
            trace!("Successfully restored DNS state");
            self.backup_writer
                .remove_backup()
                .chain_err(|| "Failed to remove backed up DNS state after restoring it")?;
        } else {
            trace!("No dns state to restore");
        }
        Ok(())
    }
}

// typedef void (WINDNS_API *WinDnsErrorSink)(const char *errorMessage, const char **details,
// uint32_t numDetails, void *context);
extern "system" fn error_sink(
    msg: *const c_char,
    detail_ptr: *const *const c_char,
    n_details: u32,
    _ctx: *mut c_void,
) {
    use std::ffi::CStr;
    if msg.is_null() {
        error!("Log message from FFI boundary is NULL");
    } else {
        if detail_ptr.is_null() || n_details == 0 {
            error!("{}", unsafe { CStr::from_ptr(msg).to_string_lossy() });
        } else {
            let raw_details = unsafe { slice::from_raw_parts(detail_ptr, n_details as usize) };
            let mut appendix = String::new();
            for detail_ptr in raw_details {
                appendix
                    .push_str(unsafe { CStr::from_ptr(*detail_ptr).to_string_lossy().borrow() });
                appendix.push_str("\n");
            }

            let message = format!(
                "{}: {}",
                unsafe { CStr::from_ptr(msg).to_string_lossy() },
                appendix
            );

            error!("{}", message);
        }
    }
}

impl Drop for WinDns {
    fn drop(&mut self) {
        if unsafe { WinDns_Deinitialize().into_result().is_ok() } {
            trace!("Successfully deinitialized WinDns");
        } else {
            error!("Failed to deinitialize WinDns");
        }
    }
}


ffi_error!(InitializationResult, ErrorKind::Initialization.into());
ffi_error!(DeinitializationResult, ErrorKind::Deinitialization.into());
ffi_error!(SettingResult, ErrorKind::Setting.into());
ffi_error!(ResettingResult, ErrorKind::Resetting.into());
ffi_error!(RecoveringResult, ErrorKind::Recovery.into());


/// A callback for writing system state data
pub extern "system" fn write_system_state_backup_cb(
    blob: *const u8,
    length: u32,
    state_writer_ptr: *mut c_void,
) -> i32 {
    let state_writer = state_writer_ptr as *mut SystemStateWriter;
    if state_writer.is_null() {
        error!("State writer pointer is null, can't save system state backup");
        return -1;
    }

    unsafe {
        trace!(
            "Writing {} bytes to store system state backup to {}",
            length,
            (*state_writer).backup_path.to_string_lossy()
        );
        let data = slice::from_raw_parts(blob, length as usize);
        match (*state_writer).write_backup(data) {
            Ok(()) => 0,
            Err(e) => {
                error!(
                    "Failed to write system state backup to {} because {}",
                    (*state_writer).backup_path.to_string_lossy(),
                    e
                );
                e.raw_os_error().unwrap_or(-1)
            }
        }
    }
}


type DNSConfigSink =
    extern "system" fn(data: *const u8, length: u32, state_writer: *mut c_void) -> i32;

// This callback can be called from multiple threads concurrently, thus if there ever is a real
// context object passed around, it should probably implement Sync.
type ErrorSink = extern "system" fn(
    msg: *const c_char,
    details: *const *const c_char,
    num_details: u32,
    ctx: *mut c_void,
);

#[allow(non_snake_case)]
extern "system" {

    #[link_name(WinDns_Initialize)]
    pub fn WinDns_Initialize(
        sink: Option<ErrorSink>,
        sink_context: *mut c_void,
    ) -> InitializationResult;

    // WinDns_Deinitialize:
    //
    // Call this function once before unloading WINDNS or exiting the process.
    #[link_name(WinDns_Deinitialize)]
    pub fn WinDns_Deinitialize() -> DeinitializationResult;

    // Configure which DNS servers should be used and start enforcing these settings.
    #[link_name(WinDns_Set)]
    pub fn WinDns_Set(
        ips: *mut *const u16,
        n_ips: u32,
        callback: Option<DNSConfigSink>,
        backup_writer: *const c_void,
    ) -> SettingResult;

    // Revert server settings to what they were before calling WinDns_Set.
    //
    // (Also taking into account external changes to DNS settings that have ocurred
    // during the period of enforcing specific settings.)
    #[link_name(WinDns_Reset)]
    pub fn WinDns_Reset() -> ResettingResult;

    #[link_name(WinDns_Recover)]
    pub fn WinDns_Recover(data: *const u8, length: u32) -> RecoveringResult;
}
