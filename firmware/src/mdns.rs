//! Supervised mDNS host and service advertisement through ESP-IDF.

use esp_idf_svc::mdns::EspMdns;
use esp_idf_svc::sys::EspError;
use log::{info, warn};

use crate::ddp::DDP_PORT;
use crate::health::{Health, ServiceState};
use crate::http::HTTP_PORT;
use crate::recovery::ExponentialBackoff;

pub struct MdnsManager {
    responder: Option<EspMdns>,
    health: Health,
    retry: ExponentialBackoff,
}

impl MdnsManager {
    pub fn new(health: Health) -> Self {
        Self {
            responder: None,
            health,
            retry: ExponentialBackoff::new(1_000, 60_000),
        }
    }

    pub fn poll(&mut self, now_ms: u64, network_ready: bool) {
        if !network_ready {
            if self.responder.is_none() {
                self.health
                    .update(|health| health.mdns = ServiceState::Stopped);
            }
            return;
        }
        if self.responder.is_some() || !self.retry.ready(now_ms) {
            return;
        }

        match start() {
            Ok(responder) => {
                self.responder = Some(responder);
                self.retry.reset();
                self.health
                    .update(|health| health.mdns = ServiceState::Running);
            }
            Err(error) => {
                let delay = self.retry.fail(now_ms);
                warn!("mdns: start failed ({error:?}); retrying in {delay} ms");
                self.health.update(|health| {
                    health.mdns = ServiceState::Retrying;
                    health.last_error = Some(format!("mDNS start failed: {error:?}"));
                });
            }
        }
    }
}

fn start() -> Result<EspMdns, EspError> {
    let mut responder = EspMdns::take()?;
    responder.set_hostname(config::HOSTNAME)?;
    responder.set_instance_name(config::NAME)?;
    responder.add_service(Some(config::NAME), "_http", "_tcp", HTTP_PORT, &[])?;
    responder.add_service(Some(config::NAME), "_slisko", "_tcp", HTTP_PORT, &[])?;
    responder.add_service(Some(config::NAME), "_ddp", "_udp", DDP_PORT, &[])?;

    info!(
        "mdns: {}.local advertises _http._tcp/{HTTP_PORT}, _slisko._tcp/{HTTP_PORT}, and _ddp._udp/{DDP_PORT} as {:?}",
        config::HOSTNAME,
        config::NAME,
    );
    Ok(responder)
}
