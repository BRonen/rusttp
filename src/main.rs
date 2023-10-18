use tokio::{
    net::TcpListener,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            socket.read(&mut buf).await.unwrap();
            
            let _buf = buf.iter()
                .filter(|a| {**a != 0})
                .map(|a| {
                    if *a == 0 {' '}
                    else {*a as char}
                }).collect::<String>();

            println!("{}", _buf);

            let status_line = "HTTP/1.1 716 OK";

            let contents = format!("{{\"test\": \"hello world\"}}\n");
            let length = contents.len();

            let response =
            format!("{status_line}\r\nContent-Type: application/json\r\nContent-Length: {length}\r\n\r\n{contents}");

            socket.write_all(response.as_bytes()).await
        });
    }
}