use crate::network::{Decorator, Header};
use std::time::Duration;

pub struct Stat {
    pub ip_address: Option<String>,
    pub http_version: Option<String>,
    pub name_lookup: Duration,
    pub connect: Duration,
    pub app_connect: Duration,
    pub pre_transfer: Duration,
    pub start_transfer: Duration,
    pub total: Duration,
    pub response_code: Option<i32>,
    pub response_headers: Vec<Header>,
    pub response_body: Vec<u8>,
}

impl Stat {
    pub fn dns_lookup(&self) -> Option<Duration> {
        Some(self.name_lookup)
    }

    pub fn tcp_handshake(&self) -> Option<Duration> {
        if self.connect > self.name_lookup {
            Some(self.connect - self.name_lookup)
        } else {
            None
        }
    }

    pub fn tls_handshake(&self) -> Option<Duration> {
        if self.app_connect > self.connect {
            Some(self.app_connect - self.connect)
        } else {
            None
        }
    }

    pub fn waiting(&self) -> Option<Duration> {
        if self.start_transfer > self.pre_transfer {
            Some(self.start_transfer - self.pre_transfer)
        } else {
            None
        }
    }

    pub fn data_transfer(&self) -> Option<Duration> {
        if self.total > self.start_transfer {
            Some(self.total - self.start_transfer)
        } else {
            None
        }
    }

    pub fn utf8_response_body(&self) -> Option<String> {
        if self.response_body.is_empty() {
            return None;
        }

        let raw = String::from_utf8_lossy(&self.response_body);
        let index = raw.find("\r\n\r\n").map(|i| i + 4).unwrap_or_default();
        let body = &raw[index..];

        Some(body.to_string())
    }
}

pub fn get_stat(handle: &mut curl::easy::Easy2<Decorator>) -> anyhow::Result<Stat> {
    let raw_headers = std::str::from_utf8(handle.get_ref().response_headers)?
        .lines()
        .map(|line| line.replace(['\r', '\n'], ""))
        .filter(|line| !line.is_empty());

    let mut headers: Vec<Header> = vec![];
    let mut http_version = None;
    let mut response_code = None;

    for header in raw_headers {
        if header.to_uppercase().starts_with("HTTP/") {
            if let Some((_, h)) = header.split_once('/') {
                let tail = h.split(' ').collect::<Vec<&str>>();
                response_code = tail.get(1).and_then(|code| code.parse().ok());
                http_version = tail.first().map(|v| v.to_string())
            }
        } else if let Some((name, value)) = header.split_once(':') {
            headers.push(Header {
                key: name.trim().to_string(),
                value: value.trim().to_string(),
            });
        }
    }

    let ip_address = handle.primary_ip()?.map(|ip| ip.to_string());

    Ok(Stat {
        ip_address,
        http_version,
        response_code,
        response_headers: headers,
        name_lookup: handle.namelookup_time()?,
        connect: handle.connect_time()?,
        app_connect: handle.appconnect_time()?,
        pre_transfer: handle.pretransfer_time()?,
        start_transfer: handle.starttransfer_time()?,
        total: handle.total_time()?,
        response_body: handle.get_ref().response_body.to_owned(),
    })
}
