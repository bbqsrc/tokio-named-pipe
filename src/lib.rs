#![cfg(windows)]

pub mod client;
pub mod config;
pub mod pipe;
pub mod secattr;
pub mod server;

pub use server::NamedPipeServerListener;
