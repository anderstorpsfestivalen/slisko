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
use log::info;

use slisko_core::controller::Controller;

use crate::ddp::DdpState;

type Shared = Arc<Mutex<Controller>>;

/// Start the HTTP control server. The returned server must be kept alive.
///
/// mDNS discovery (`slisko._http._tcp`) is a TODO: it needs the ESP-IDF `mdns`
/// managed component enabled in the build (`esp_idf_svc::mdns` is cfg-gated on
/// it). For now the board is reached by its DHCP IP.
pub fn start(ctrl: Shared, ddp: Arc<DdpState>) -> Result<EspHttpServer<'static>, EspIOError> {
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

    let c = ctrl.clone();
    server.fn_handler("/enable", Method::Get, move |req| {
        let msg = match query_param(req.uri(), "p") {
            Some(name) => {
                c.lock().unwrap().enable(&name);
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
                c.lock().unwrap().disable(&name);
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
    let c = ctrl.lock().unwrap();
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
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(v.to_string());
            }
        }
    }
    None
}

const INDEX_HTML: &str = "<!doctype html><meta charset=utf-8><title>slisko</title>\
<h1>slisko</h1><p>Endpoints: \
<code>/patterns</code>, <code>/enable?p=NAME</code>, \
<code>/disable?p=NAME</code>, <code>/source?mode=internal|ddp</code></p>";
