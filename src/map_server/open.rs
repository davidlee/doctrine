// SPDX-License-Identifier: GPL-3.0-only
//! Map-server URL construction and browser-open helpers (SL-072 PHASE-06).
//!
//! Engine-tier: `map_url` is pure and testable without binding a socket;
//! `open_browser` is impure (the sole shell-out seam in this module).

use std::net::SocketAddr;

/// Construct the browser URL. Pure — testable without binding a socket.
pub(crate) fn map_url(addr: SocketAddr, focus: Option<&str>, depth: u8) -> String {
    let base = format!("http://{addr}/");
    let Some(focus_id) = focus else {
        return base;
    };
    if depth == 1 {
        format!("{base}#/focus/{focus_id}")
    } else {
        format!("{base}#/focus/{focus_id}?depth={depth}")
    }
}

/// Open the browser at the given URL. Non-fatal — caller logs errors.
pub(crate) fn open_browser(url: &str) -> anyhow::Result<()> {
    webbrowser::open(url).map_err(Into::into)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn url_no_focus() {
        let addr = SocketAddr::from_str("127.0.0.1:8080").unwrap();
        assert_eq!(map_url(addr, None, 1), "http://127.0.0.1:8080/");
    }

    #[test]
    fn url_focus_depth_default() {
        let addr = SocketAddr::from_str("127.0.0.1:8080").unwrap();
        assert_eq!(
            map_url(addr, Some("SL-001"), 1),
            "http://127.0.0.1:8080/#/focus/SL-001"
        );
    }

    #[test]
    fn url_focus_depth_2() {
        let addr = SocketAddr::from_str("127.0.0.1:8080").unwrap();
        assert_eq!(
            map_url(addr, Some("SL-001"), 2),
            "http://127.0.0.1:8080/#/focus/SL-001?depth=2"
        );
    }

    #[test]
    fn url_ipv6_no_focus() {
        let addr = SocketAddr::from_str("[::1]:8080").unwrap();
        assert_eq!(map_url(addr, None, 1), "http://[::1]:8080/");
    }
}
