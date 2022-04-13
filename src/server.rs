use std::{
    ffi::c_void,
    path::{Path, PathBuf},
};

use futures::Stream;
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

use crate::{config::NamedPipeConfig, pipe::NamedPipe};

pub struct NamedPipeServerListener {
    path: PathBuf,
    config: NamedPipeConfig,
}

impl NamedPipeServerListener {
    pub fn listen(
        self,
    ) -> std::io::Result<impl Stream<Item = std::io::Result<NamedPipe>> + 'static> {
        let pipe = listen(&self.path, &self.config, true)?;

        let stream = futures::stream::try_unfold((pipe, self), |(pipe, this)| async move {
            let () = pipe.connect().await?;
            let conn = NamedPipe::Server(pipe);
            let pipe = listen(&this.path, &this.config, false)?;
            Ok(Some((conn, (pipe, this))))
        });

        Ok(stream)
    }

    pub fn bind(
        path: PathBuf,
        config: NamedPipeConfig,
    ) -> std::io::Result<impl Stream<Item = std::io::Result<NamedPipe>> + 'static> {
        NamedPipeServerListener { path, config }.listen()
    }
}

#[inline]
fn listen(
    path: &Path,
    config: &NamedPipeConfig,
    is_first_instance: bool,
) -> std::io::Result<NamedPipeServer> {
    unsafe {
        ServerOptions::new()
            .first_pipe_instance(is_first_instance)
            .reject_remote_clients(config.reject_remote_clients)
            .access_inbound(config.inbound)
            .access_outbound(config.outbound)
            .in_buffer_size(config.in_buffer_size)
            .out_buffer_size(config.out_buffer_size)
            .create_with_security_attributes_raw(
                &path,
                std::ptr::addr_of!(config.security_attributes) as *mut c_void,
            )
    }
}
