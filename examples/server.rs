use futures::{StreamExt, pin_mut};
use tokio::io::AsyncReadExt;
use tokio_named_pipe::NamedPipeServerListener;

#[tokio::main]
async fn main() {
    let pipe = NamedPipeServerListener::bind("//./pipe/test-pipe".into(), Default::default()).unwrap();

    pin_mut!(pipe);

    for mut connection in pipe.next().await.unwrap() {
        let mut buffer = String::new();
        connection.read_to_string(&mut buffer).await.unwrap();
        println!("GOT: {}", buffer);
    }
}