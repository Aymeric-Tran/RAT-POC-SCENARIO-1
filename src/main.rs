use std::net::TcpStream;

fn main() {
    if let Ok(_stream) = TcpStream::connect("127.0.0.1:8000") {
        println!("Connecté au serveur");
    } else {
        println!("Impossible de se connecter au serveur");
    }
}
