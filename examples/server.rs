use futures::{StreamExt, pin_mut};
use tokio::io::AsyncReadExt;
use tokio_named_pipe::NamedPipeServerListener;

#[tokio::main]
async fn main() {
    let mut descriptor = tokio_named_pipe::secattr::SecurityDescriptor::world().unwrap();
    let mut config = tokio_named_pipe::config::NamedPipeConfig::default();
    config.security_attributes = tokio_named_pipe::secattr::SecurityAttributes::new(&mut descriptor, false);
    let pipe = NamedPipeServerListener::bind("//./pipe/test-pipe".into(), config).unwrap();

    pin_mut!(pipe);

    for mut connection in pipe.next().await.unwrap() {
        let mut buffer = String::new();
        connection.read_to_string(&mut buffer).await.unwrap();
        println!("GOT: {}", buffer);
    }
}
