use std::io::Write;
use std::time::Duration;

use crate::network::Config;
use crate::network::Stat;

struct NetworkEvent<'a> {
    name: &'a str,
    duration: Duration,
}

impl<'a> NetworkEvent<'a> {
    fn dns_lookup(stat: &Stat) -> Option<Self> {
        stat.dns_lookup().map(|duration| Self {
            name: "DNS Lookup",
            duration,
        })
    }

    fn tcp_handshake(stat: &Stat) -> Option<Self> {
        stat.tcp_handshake().map(|duration| Self {
            name: "TCP Handshake",
            duration,
        })
    }

    fn tls_handshake(stat: &Stat) -> Option<Self> {
        stat.tls_handshake().map(|duration| Self {
            name: "TLS Handshake",
            duration,
        })
    }

    fn server_processing(stat: &Stat) -> Option<Self> {
        stat.waiting().map(|duration| Self {
            name: "Server Processing",
            duration,
        })
    }

    fn data_transfer(stat: &Stat) -> Option<Self> {
        stat.data_transfer().map(|duration| Self {
            name: "Data Transfer",
            duration,
        })
    }

    fn total(stat: &Stat) -> Option<Self> {
        Some(Self {
            name: "Total",
            duration: stat.total,
        })
    }

    fn name_lookup(stat: &Stat) -> Option<Self> {
        Some(Self {
            name: "Name Lookup",
            duration: stat.name_lookup,
        })
    }

    fn connect(stat: &Stat) -> Option<Self> {
        Some(Self {
            name: "Connect",
            duration: stat.connect,
        })
    }

    fn app_connect(stat: &Stat) -> Option<Self> {
        stat.tls_handshake().map(|_| Self {
            name: "App Connect",
            duration: stat.app_connect,
        })
    }

    fn pre_transfer(stat: &Stat) -> Option<Self> {
        Some(Self {
            name: "Pre Transfer",
            duration: stat.pre_transfer,
        })
    }

    fn start_transfer(stat: &Stat) -> Option<Self> {
        Some(Self {
            name: "Start Transfer",
            duration: stat.start_transfer,
        })
    }
}

/// Screen is a struct that represents the screen output.
///
/// # Example
///
/// ```rust
/// use cetar::network::{Config, Stat};
/// use cetar::output::Ui;
/// use std::time::Duration;
///
/// let config = Config::default();
/// let stat = Stat::default();
///
/// let screen = Screen::new(&config, &stat);
/// screen.display();
/// ```
pub struct Screen<'a> {
    config: &'a Config<'a>,
    stat: &'a Stat,
}

impl<'a> Screen<'a> {
    const PADDING: usize = 35;
    const MAX_PADDING: usize = 50;

    pub fn new(config: &'a Config<'a>, stat: &'a Stat) -> Self {
        Self { config, stat }
    }

    #[inline]
    fn scale_factor(&self) -> f64 {
        match self.stat.total.as_millis() {
            0..=100 => 1.0,
            101..=500 => 5.0,
            501..=1000 => 10.0,
            1001..=5000 => 50.0,
            5001..=10000 => 100.0,
            _ => 1000.0,
        }
    }

    fn event_bar(&self, event: &NetworkEvent) -> String {
        let duration_ms = event.duration.as_millis();
        let bar_length = (duration_ms as f64 / self.scale_factor()) as usize;
        "█".repeat(bar_length)
    }

    fn display_events(&self, events: &[Option<NetworkEvent>]) {
        for event in events.iter().flatten() {
            println!(
                "{name:<width$} {bar} {duration_ms}ms",
                name = self.config.color.paint(event.name),
                duration_ms = event.duration.as_millis(),
                bar = self.event_bar(event),
                width = Self::PADDING
            );
        }
    }

    fn display_network_timings(&self) {
        println!("Network Timings:");

        let events = &[
            NetworkEvent::dns_lookup(self.stat),
            NetworkEvent::tcp_handshake(self.stat),
            NetworkEvent::tls_handshake(self.stat),
            NetworkEvent::server_processing(self.stat),
            NetworkEvent::data_transfer(self.stat),
        ];

        self.display_events(events);
    }

    fn display_detailed_timings(&self) {
        println!("Detailed Timings:");

        let events = &[
            NetworkEvent::name_lookup(self.stat),
            NetworkEvent::connect(self.stat),
            NetworkEvent::app_connect(self.stat),
            NetworkEvent::pre_transfer(self.stat),
            NetworkEvent::start_transfer(self.stat),
            NetworkEvent::total(self.stat),
        ];

        self.display_events(events);
    }

    fn display_response_headers(&self) {
        println!();
        println!(
            "HTTP/{} {}",
            self.stat
                .http_version
                .as_ref()
                .unwrap_or(&"Unknown".to_string()),
            self.stat.response_status_code.unwrap_or_default()
        );

        let max_name_len = self
            .stat
            .response_headers
            .iter()
            .map(|header| header.key.len())
            .max()
            .unwrap_or(Self::PADDING);

        let width = if max_name_len > Self::MAX_PADDING {
            Self::MAX_PADDING
        } else if max_name_len < Self::PADDING {
            Self::PADDING
        } else {
            max_name_len
        };

        for header in &self.stat.response_headers {
            println!(
                "{key:<width$} {value:<width$}",
                key = self.config.color.paint(header.header_key().as_str()),
                value = header.value,
                width = width
            );
        }
    }

    fn display_response_body(&self) {
        if let Some(body) = self.stat.utf8_response_body() {
            println!("Response Body:");
            println!();
            println!("{}", self.config.color.paint(&body));
        }
    }

    /// Display the screen output.
    ///
    pub fn display(&self) {
        println!();
        println!(
            "Connect {}",
            self.config.color.paint(
                self.stat
                    .ip_address
                    .as_ref()
                    .unwrap_or(&"Unknown".to_string())
            )
        );
        println!();
        self.display_network_timings();
        println!();
        self.display_detailed_timings();
        if self.config.display_response_headers {
            println!();
            self.display_response_headers();
        }
        if self.config.display_response_body {
            println!();
            self.display_response_body();
        }
    }
}

/// Handle the output of the request.
///
/// # Example
///
/// ```rust
/// use cetar::network::{Config, Stat};
/// use cetar::output::handle_output;
///
/// let config = Config::default();
/// let stat = Stat::default();
///
/// handle_output(&config, &stat).unwrap();
///
/// ```
pub fn handle_output(config: &Config, stat: &Stat) -> anyhow::Result<()> {
    if let Some(output) = &config.output {
        if let Some(body) = stat.utf8_response_body() {
            let mut file = std::fs::File::create(output.as_ref())?;
            file.write_all(body.as_bytes())?;
        }
    }

    Ok(())
}
