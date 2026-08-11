//! Small HTTP control surface (mirrors slisko's REST API) + mDNS discovery.
//!
//! Endpoints (all GET, query-string args so no path-param routing needed):
//!   /            — tiny status/help page
//!   /patterns    — JSON list of {name, category, enabled}
//!   /enable?p=   — enable a pattern
//!   /disable?p=  — disable a pattern
//!   /source?mode=internal|ddp — switch the pixel source
//!
//! mDNS advertises `slisko._http._tcp` on port 80 for discovery.

use std::sync::{Arc, Mutex};

use esp_idf_svc::http::Method;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::{EspIOError, Write};
use esp_idf_svc::sys::esp_timer_get_time;
use log::{info, warn};

use engine::controller::Controller;

use crate::ddp::DdpState;
use crate::health::{Health, ServiceState, lock_recover};
use crate::recovery::ExponentialBackoff;

type Shared = Arc<Mutex<Controller>>;

pub struct HttpManager {
    server: Option<EspHttpServer<'static>>,
    ctrl: Shared,
    ddp: Arc<DdpState>,
    health: Health,
    retry: ExponentialBackoff,
}

impl HttpManager {
    pub fn new(ctrl: Shared, ddp: Arc<DdpState>, health: Health) -> Self {
        Self {
            server: None,
            ctrl,
            ddp,
            health,
            retry: ExponentialBackoff::new(1_000, 60_000),
        }
    }

    pub fn poll(&mut self, now_ms: u64, network_ready: bool) {
        // EspHttpServer ultimately creates lwIP sockets. On ESP-IDF that can
        // assert rather than return an error when esp-netif construction has
        // failed, so wait for a valid DHCP/IP state before first startup.
        if !network_ready {
            if self.server.is_none() {
                self.health
                    .update(|health| health.http = ServiceState::Stopped);
            }
            return;
        }
        if self.server.is_some() || !self.retry.ready(now_ms) {
            return;
        }
        match start(self.ctrl.clone(), self.ddp.clone(), self.health.clone()) {
            Ok(server) => {
                self.server = Some(server);
                self.retry.reset();
                self.health
                    .update(|health| health.http = ServiceState::Running);
            }
            Err(error) => {
                let delay = self.retry.fail(now_ms);
                warn!("http: start failed ({error:?}); retrying in {delay} ms");
                self.health.update(|health| {
                    health.http = ServiceState::Retrying;
                    health.last_error = Some(format!("HTTP start failed: {error:?}"));
                });
            }
        }
    }
}

/// Start the HTTP control server. The returned server must be kept alive.
fn start(
    ctrl: Shared,
    ddp: Arc<DdpState>,
    health: Health,
) -> Result<EspHttpServer<'static>, EspIOError> {
    let mut server = EspHttpServer::new(&Configuration::default())?;

    server.fn_handler("/", Method::Get, |req| {
        let mut resp = req.into_ok_response()?;
        resp.write_all(INDEX_HTML.as_bytes())?;
        Ok::<(), esp_idf_svc::io::EspIOError>(())
    })?;

    let c = ctrl.clone();
    server.fn_handler("/patterns", Method::Get, move |req| {
        let body = patterns_json(&c);
        let mut resp = req.into_ok_response()?;
        resp.write_all(body.as_bytes())?;
        Ok::<(), esp_idf_svc::io::EspIOError>(())
    })?;

    let h = health.clone();
    server.fn_handler("/health", Method::Get, move |req| {
        let body = h.snapshot().to_json(monotonic_ms());
        let mut resp = req.into_response(
            200,
            Some("OK"),
            &[("Content-Type", "application/json")],
        )?;
        resp.write_all(body.as_bytes())?;
        Ok::<(), esp_idf_svc::io::EspIOError>(())
    })?;

    let c = ctrl.clone();
    server.fn_handler("/enable", Method::Get, move |req| {
        let msg = match query_param(req.uri(), "p") {
            Some(name) => {
                lock_recover(&c).enable(&name);
                format!("enabled {name}")
            }
            None => "missing ?p=<pattern>".into(),
        };
        let mut resp = req.into_ok_response()?;
        resp.write_all(msg.as_bytes())?;
        Ok::<(), esp_idf_svc::io::EspIOError>(())
    })?;

    let c = ctrl.clone();
    server.fn_handler("/disable", Method::Get, move |req| {
        let msg = match query_param(req.uri(), "p") {
            Some(name) => {
                lock_recover(&c).disable(&name);
                format!("disabled {name}")
            }
            None => "missing ?p=<pattern>".into(),
        };
        let mut resp = req.into_ok_response()?;
        resp.write_all(msg.as_bytes())?;
        Ok::<(), esp_idf_svc::io::EspIOError>(())
    })?;

    let d = ddp.clone();
    server.fn_handler("/source", Method::Get, move |req| {
        let msg = match query_param(req.uri(), "mode").as_deref() {
            Some("ddp") => {
                d.set_enabled(true);
                "source = ddp (external override when frames arrive)"
            }
            Some("internal") => {
                d.set_enabled(false);
                "source = internal patterns"
            }
            _ => "use ?mode=internal|ddp",
        };
        let mut resp = req.into_ok_response()?;
        resp.write_all(msg.as_bytes())?;
        Ok::<(), esp_idf_svc::io::EspIOError>(())
    })?;

    info!("http: control server up on :80");
    Ok(server)
}

/// Build the `/patterns` JSON without pulling in a serializer.
fn patterns_json(ctrl: &Shared) -> String {
    let c = lock_recover(ctrl);
    let mut s = String::from("[");
    for (i, (info, enabled)) in c.pattern_list().iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"category\":\"{}\",\"enabled\":{}}}",
            info.name, info.category, enabled
        ));
    }
    s.push(']');
    s
}

/// Extract `key`'s value from a `path?a=b&key=val` URI.
fn query_param(uri: &str, key: &str) -> Option<String> {
    let q = uri.split_once('?')?.1;
    for pair in q.split('&') {
        if let Some((k, v)) = pair.split_once('=')
            && k == key
        {
            return Some(v.to_string());
        }
    }
    None
}

const INDEX_HTML: &str = "<!doctype html><meta charset=utf-8><title>slisko</title>\
<h1>slisko</h1><p>Endpoints: \
<code>/health</code>, <code>/patterns</code>, <code>/enable?p=NAME</code>, \
<code>/disable?p=NAME</code>, <code>/source?mode=internal|ddp</code></p>";

fn monotonic_ms() -> u64 {
    unsafe { esp_timer_get_time().max(0) as u64 / 1_000 }
}
