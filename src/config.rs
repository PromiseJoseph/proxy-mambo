use crate::types::{Config, Upstream};

impl Config {
    pub fn new() -> Self {
        Self {
            proxy_addr: "127.0.0.1:8080".to_string(),
            upstream_addr: vec![Upstream {
                id: "default".to_string(),
                addr: "127.0.0.1:8000".to_string(),
            }],
            max_header_size: 16384, // Default maximum header size set to 16KiB
        }
    }

    pub fn add_upstream(&mut self, upstream: Upstream) {
        self.upstream_addr.push(Upstream {
            id: upstream.id,
            addr: upstream.addr,
        });
    }

    pub fn get_upstream(&self, id: &str) -> Option<&Upstream> {
        if let Some(upstream) = self.upstream_addr.iter().find(|u| u.id == id) {
            Some(upstream)
        } else {
            None
        }
    }

    pub fn set_proxy_addr(&mut self, addr: &str) {
        self.proxy_addr = addr.to_string();
    }

    pub fn set_max_header_size(&mut self, size: usize) {
        self.max_header_size = size;
    }

    pub fn get_proxy_addr(&self) -> &str {
        &self.proxy_addr
    }

    pub fn get_max_header_size(&self) -> usize {
        self.max_header_size
    }
}
