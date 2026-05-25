pub struct UdpRadiusConnectorConfig {
    buffer_size: usize,
    address: String,
    port: u16,
}

impl UdpRadiusConnectorConfig {
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn with_buffer_size(&mut self, buffer_size: usize) -> &mut Self {
        self.buffer_size = buffer_size;
        self
    }

    pub fn with_address(&mut self, address: String) -> &mut Self {
        self.address = address;
        self
    }

    pub fn with_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }
}

impl Default for UdpRadiusConnectorConfig {
    fn default() -> Self {
        Self {
            buffer_size: 4096,
            address: "127.0.0.1".to_string(),
            port: 1812,
        }
    }
}
