use crate::secattr::SecurityAttributes;

pub struct NamedPipeConfig {
    pub reject_remote_clients: bool,
    pub inbound: bool,
    pub outbound: bool,
    pub out_buffer_size: u32,
    pub in_buffer_size: u32,
    pub security_attributes: SecurityAttributes,
}

impl Default for NamedPipeConfig {
    fn default() -> Self {
        NamedPipeConfig {
            reject_remote_clients: true,
            inbound: true,
            outbound: true,
            out_buffer_size: 0x10000,
            in_buffer_size: 0x10000,
            security_attributes: Default::default(),
        }
    }
}
