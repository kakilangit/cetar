use std::borrow::Cow;
use std::io::Read;

use crate::color::Color;
use crate::stat::{get_stat, Stat};
use crate::{make_color, print_error};

//
// https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods
//
#[derive(Debug, Clone)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

impl<'a> From<&'a Method> for &'a str {
    fn from(method: &'a Method) -> &'a str {
        match method {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Connect => "CONNECT",
            Method::Options => "OPTIONS",
            Method::Trace => "TRACE",
            Method::Patch => "PATCH",
        }
    }
}

impl TryFrom<String> for Method {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "HEAD" => Ok(Self::Head),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "CONNECT" => Ok(Self::Connect),
            "OPTIONS" => Ok(Self::Options),
            "TRACE" => Ok(Self::Trace),
            "PATCH" => Ok(Self::Patch),
            _ => Err(anyhow::anyhow!("Invalid method, please use GET, HEAD, POST, PUT, DELETE, CONNECT, OPTIONS, TRACE, PATCH")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Header {
    pub key: String,
    pub value: String,
}

impl Header {
    pub fn header_key(&self) -> String {
        self.key
            .split('-')
            .map(|word| {
                word.chars()
                    .enumerate()
                    .map(|(i, c)| {
                        if i == 0 {
                            c.to_uppercase().to_string()
                        } else {
                            c.to_lowercase().to_string()
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("-")
    }
}

impl core::fmt::Display for Header {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

impl std::str::FromStr for Header {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(':') {
            Some((key, value)) => Ok(Self {
                key: key.trim().to_string(),
                value: value.trim().to_string(),
            }),
            None => anyhow::bail!("Invalid header format, please use key: value"),
        }
    }
}

pub struct Config<'a> {
    pub url: Cow<'a, str>,
    pub method: Method,
    pub color: Color,
    pub request_headers: Vec<Header>,
    pub request_body: Option<Cow<'a, str>>,
    pub output: Option<Cow<'a, str>>,
    pub display_response_body: bool,
    pub display_response_headers: bool,
    pub follow_redirects: bool,
    pub verbose: bool,
}

pub struct Decorator<'a> {
    config: &'a Config<'a>,
    pub response_headers: &'a mut Vec<u8>,
    pub response_body: &'a mut Vec<u8>,
}

impl<'a> Decorator<'a> {
    pub fn new(
        config: &'a Config<'a>,
        response_headers: &'a mut Vec<u8>,
        response_body: &'a mut Vec<u8>,
    ) -> Self {
        Self {
            config,
            response_headers,
            response_body,
        }
    }
}

impl<'a> curl::easy::Handler for Decorator<'a> {
    fn header(&mut self, data: &[u8]) -> bool {
        self.response_headers.extend_from_slice(data);
        true
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize, curl::easy::ReadError> {
        match &self.config.request_body {
            Some(d) => match d.as_bytes().read(data) {
                Ok(len) => Ok(len),
                Err(e) => {
                    print_error!("Error reading data: {}", e);
                    Err(curl::easy::ReadError::Abort)
                }
            },
            None => Ok(0),
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, curl::easy::WriteError> {
        self.response_body.extend_from_slice(data);
        Ok(data.len())
    }
}

pub fn send_request(conf: &Config) -> anyhow::Result<Stat> {
    let mut headers = Vec::new();
    let mut response = Vec::new();
    let mut easy = curl::easy::Easy2::new(Decorator::new(conf, &mut headers, &mut response));

    easy.url(&conf.url)?;
    easy.show_header(true)?;
    easy.follow_location(conf.follow_redirects)?;
    easy.verbose(conf.verbose)?;

    if !conf.request_headers.is_empty() {
        let mut headers = curl::easy::List::new();
        for header in &conf.request_headers {
            headers.append(&header.to_string())?;
        }
        easy.http_headers(headers)?;
    }

    let data_size = conf.request_body.as_ref().map(|d| d.len() as u64);

    match &conf.method {
        Method::Get => easy.get(true)?,
        Method::Head => easy.nobody(true)?,
        Method::Post => {
            easy.post(true)?;
            if let Some(ds) = data_size {
                easy.post_field_size(ds)?;
            }
        }
        Method::Patch => {
            easy.custom_request("PATCH")?;
            if let Some(ds) = data_size {
                easy.post_field_size(ds)?;
            }
        }
        Method::Put => {
            easy.put(true)?;
            if let Some(ds) = data_size {
                easy.in_filesize(ds)?;
            }
        }
        _ => easy.custom_request((&conf.method).into())?,
    }

    easy.perform()?;

    get_stat(&mut easy)
}
