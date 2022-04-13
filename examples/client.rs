use tokio::net::windows::named_pipe;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    let client = named_pipe::ClientOptions::new();
    let mut client = client.open("//./pipe/test-pipe").unwrap();

    client.write(b"hello everyone").await.unwrap();
}