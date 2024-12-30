use tokio::net::TcpStream;
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

const C2_ADDR: &str = "192.168.9.136:8080";

#[tokio::main]
async fn main() -> io::Result<()> {
    let server_address = C2_ADDR;
    let mut stream = TcpStream::connect(server_address).await?;

    println!("Connected to the server at {}", server_address);

    loop {
        let mut reader = BufReader::new(io::stdin());

        println!("Enter some text :");

        let mut msg = String::new();
        
        reader.read_line(&mut msg).await?;

        stream.write_all(msg.as_bytes()).await?;
    
        // let mut buffer = [0; 1024];
        // let n = stream.read(&mut buffer).await?;
        
        // let received = match std::str::from_utf8(&buffer[..n]) {
        //     Ok(v) => v,
        //     Err(e) => {
        //         println!("Failed {}", e);
        //         return Ok(())
        //     }
        // };
    
        // println!("Received : {}", received);
    }
}
