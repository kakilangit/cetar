use std::borrow::Cow;
use std::io::Read;
use std::time::Duration;

use crate::color::Color;
use crate::{make_color, print_error};

/// Configuration struct for the network module
///
/// # Example
///
/// ```rust
/// use cetar::network::{Config, Method};
/// use std::borrow::Cow;
///
/// let config = Config {
///     url: Cow::Borrowed("https://example.com"),
///     method: Method::Get,
///     color: "cyan".into(),
///     request_headers: vec![],
///     request_body: None,
///     output: None,
///     display_response_body: false,
///     display_response_headers: false,
///     follow_redirects: false,
///     verbose: false,
/// };
/// ```
///
#[derive(Default)]
pub struct Config<'a> {
    /// URL to send the request
    pub url: Cow<'a, str>,
    /// HTTP method to use
    pub method: Method,
    /// Main output color
    pub color: Color,
    /// Request headers
    pub request_headers: Vec<Header>,
    /// Request body
    pub request_body: Option<Cow<'a, str>>,
    /// Write output to file
    pub output: Option<Cow<'a, str>>,
    /// Display response body
    pub display_response_body: bool,
    /// Display response headers
    pub display_response_headers: bool,
    /// Follow HTTP 3xx redirects
    pub follow_redirects: bool,
    /// Verbose output
    pub verbose: bool,
}

/// Implements decorator pattern for Easy2 CURL calls
///
/// # Example
///
/// ```rust
/// use cetar::network::{Config, Decorator, Header};
/// use std::borrow::Cow;
///
/// let config = Config {
///    url: Cow::Borrowed("https://example.com"),
///    ..Default::default()
/// };
///
/// let mut response_headers = vec![];
/// let mut response_body = vec![];
/// let decorator = Decorator::new(&config, &mut response_headers, &mut response_body);
/// let handler = curl::easy::Easy2::new(decorator);
/// ```
pub struct Decorator<'a> {
    config: &'a Config<'a>,
    /// Placeholder for response headers
    pub response_headers: &'a mut Vec<u8>,
    /// Placeholder for response body
    pub response_body: &'a mut Vec<u8>,
}

impl<'a> Decorator<'a> {
    /// Create a new Decorator instance
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

/// Header struct to store key-value pairs
///
/// # Example
///
/// ```rust
/// use cetar::network::Header;
/// use std::str::FromStr;
///
/// let header = Header::from_str("Content-Type: application/json").unwrap();
///
/// assert_eq!(header.key, "Content-Type");
/// assert_eq!(header.value, "application/json");
/// ```
///
#[derive(Clone, Debug)]
pub struct Header {
    /// Header key
    pub key: String,
    /// Header value
    pub value: String,
}

impl Header {
    /// Get the header key in a format suitable for HTTP headers
    /// e.g. `content-type` -> `Content-Type`
    ///
    /// # Example
    ///
    /// ```rust
    /// use cetar::network::Header;
    ///
    /// let header = Header {
    ///     key: "content-type".to_string(),
    ///     value: "application/json".to_string(),
    /// };
    ///
    /// assert_eq!(header.header_key(), "Content-Type");
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

/// Stat struct to store network statistics
///
/// # Example
///
/// ```rust
/// use cetar::network::{Header, Stat};
/// use std::time::Duration;
///
/// let stat = Stat {
///     ip_address: Some("127.0.0.1".to_string()),
///     http_version: Some("HTTP/1.1".to_string()),
///     name_lookup: Duration::from_millis(100),
///     connect: Duration::from_millis(200),
///     app_connect: Duration::from_millis(300),
///     pre_transfer: Duration::from_millis(400),
///     start_transfer: Duration::from_millis(500),
///     total: Duration::from_millis(600),
///     response_status_code: Some(200),
///     response_headers: vec![Header::from_str("Content-Type: application/json").unwrap()],
///     response_body: vec![],
/// };
///
/// assert_eq!(stat.dns_lookup(), Some(Duration::from_millis(100)));
/// assert_eq!(stat.tcp_handshake(), Some(Duration::from_millis(100)));
/// assert_eq!(stat.tls_handshake(), Some(Duration::from_millis(100)));
/// assert_eq!(stat.server_processing(), Some(Duration::from_millis(100)));
/// assert_eq!(stat.content_transfer(), Some(Duration::from_millis(100)));
/// ```
///
#[derive(Default)]
pub struct Stat {
    /// IP address of the server
    pub ip_address: Option<String>,
    /// HTTP version used
    pub http_version: Option<String>,
    /// DNS lookup time
    pub name_lookup: Duration,
    /// TCP handshake time
    pub connect: Duration,
    /// TLS handshake time
    pub app_connect: Duration,
    /// Time to start the transfer
    pub pre_transfer: Duration,
    /// Time to start receiving the response
    pub start_transfer: Duration,
    /// Total time taken
    pub total: Duration,
    /// Response status code
    pub response_status_code: Option<i32>,
    /// Response headers
    pub response_headers: Vec<Header>,
    /// Response body
    pub response_body: Vec<u8>,
}

impl Stat {
    /// Get the DNS lookup time
    pub fn dns_lookup(&self) -> Option<Duration> {
        Some(self.name_lookup)
    }

    /// Get the TCP handshake time
    pub fn tcp_handshake(&self) -> Option<Duration> {
        if self.connect > self.name_lookup {
            Some(self.connect - self.name_lookup)
        } else {
            None
        }
    }

    /// Get the TLS handshake time, if applicable
    pub fn tls_handshake(&self) -> Option<Duration> {
        if self.app_connect > self.connect {
            Some(self.app_connect - self.connect)
        } else {
            None
        }
    }

    /// Get the server processing time
    pub fn waiting(&self) -> Option<Duration> {
        if self.start_transfer > self.pre_transfer {
            Some(self.start_transfer - self.pre_transfer)
        } else {
            None
        }
    }

    /// Get the content transfer time
    pub fn data_transfer(&self) -> Option<Duration> {
        if self.total > self.start_transfer {
            Some(self.total - self.start_transfer)
        } else {
            None
        }
    }

    /// Convert the response body to a UTF-8 string
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

impl<'a> TryFrom<&mut curl::easy::Easy2<Decorator<'a>>> for Stat {
    type Error = anyhow::Error;

    fn try_from(handle: &mut curl::easy::Easy2<Decorator<'a>>) -> Result<Self, Self::Error> {
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
            response_status_code: response_code,
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
}

/// Enum for HTTP methods based on <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods>
///
/// # Example
///
/// ```rust
/// use cetar::network::Method;
/// use std::convert::TryFrom;
///
/// let get = Method::Get;
/// let post = Method::try_from("POST".to_string()).unwrap();
///
/// assert_eq!(get, Method::Get);
/// assert_eq!(post, Method::Post);
/// ```
///
#[derive(Debug, Clone)]
pub enum Method {
    /// The GET method requests a representation of the specified resource. Requests using GET should only retrieve data.
    Get,
    /// The HEAD method asks for a response identical to that of a GET request, but without the response body.
    Head,
    /// The POST method is used to submit an entity to the specified resource, often causing a change in state or side effects on the server.
    Post,
    /// The PUT method replaces all current representations of the target resource with the request payload.
    Put,
    /// The DELETE method deletes the specified resource.
    Delete,
    /// The CONNECT method establishes a tunnel to the server identified by the target resource.
    Connect,
    /// The OPTIONS method is used to describe the communication options for the target resource.
    Options,
    /// The TRACE method performs a message loop-back test along the path to the target resource.
    Trace,
    /// The PATCH method is used to apply partial modifications to a resource.
    Patch,
}

impl Default for Method {
    fn default() -> Self {
        Self::Get
    }
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

/// Send a request to the specified URL and return the `Stat` struct with the response information.
/// The `Config` struct contains the configuration for the request, such as the URL, method, headers, etc
///
/// # Arguments
///
/// * `conf` - The configuration for the request.
///
/// # Returns
///
/// A `Stat` struct with the response information, such as the status code, response headers, response body, etc
///
/// # Example
///
/// ```rust
/// use cetar::network::{send_request, Config};
///
/// let conf = Config {
///    url: "https://httpbin.org/get".into(),
///    ..Default::default()
/// };
///
/// let stat = send_request(&conf).unwrap();
///
/// println!("Status code: {}", stat.response_status_code);
/// ```
///
pub fn send_request(conf: &Config) -> anyhow::Result<Stat> {
    let mut headers = vec![];
    let mut response = vec![];
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

    Stat::try_from(&mut easy)
}
