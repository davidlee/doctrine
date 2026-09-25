// SPDX-License-Identifier: GPL-3.0-only
//! The `[design]` table of `doctrine.toml` — how the inquiry map reaches the
//! user (SL-266 `DEC-309`).
//!
//! The shared `doctrine.toml` parse keeps `[design]` raw, so a malformed entry
//! never fails an unrelated reader (the lazy `[estimation]` precedent). The
//! design writes call [`resolve_map_delivery`] before writing, and it refuses
//! every malformation — a misspelt key included, which would otherwise read as
//! absent and silently select [`MapDelivery::Relay`] (STD-003).

/// The one key the `[design]` table accepts.
pub(crate) const MAP_DELIVERY_KEY: &str = "map_delivery";
/// The agent pastes `doctrine design tree` output after a map change.
pub(crate) const MAP_DELIVERY_RELAY: &str = "relay";
/// The user watches the tree in a pane of their own; the agent stays quiet.
pub(crate) const MAP_DELIVERY_SIDECAR: &str = "sidecar";

/// Who puts the map in front of the user.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MapDelivery {
    /// The write that changed the map tells the agent to show the tree.
    #[default]
    Relay,
    /// The user runs `doctrine design tree` themselves.
    Sidecar,
}

/// Resolve the raw `[design]` entry. Absent entry or key → [`MapDelivery::Relay`];
/// anything else that is not `map_delivery = "relay" | "sidecar"` is refused,
/// naming what is accepted.
pub(crate) fn resolve_map_delivery(design: Option<&toml::Value>) -> anyhow::Result<MapDelivery> {
    let accepted = || {
        format!(
            "accepted: `[design] {MAP_DELIVERY_KEY} = \"{MAP_DELIVERY_RELAY}\" | \"{MAP_DELIVERY_SIDECAR}\"`"
        )
    };
    let Some(design) = design else {
        return Ok(MapDelivery::default());
    };
    let Some(table) = design.as_table() else {
        anyhow::bail!("doctrine.toml: `design` is not a table — {}", accepted());
    };
    if let Some(unknown) = table.keys().find(|key| *key != MAP_DELIVERY_KEY) {
        anyhow::bail!(
            "doctrine.toml: unknown key `[design] {unknown}` — {}",
            accepted()
        );
    }
    match table.get(MAP_DELIVERY_KEY) {
        None => Ok(MapDelivery::default()),
        Some(toml::Value::String(value)) if value == MAP_DELIVERY_RELAY => Ok(MapDelivery::Relay),
        Some(toml::Value::String(value)) if value == MAP_DELIVERY_SIDECAR => {
            Ok(MapDelivery::Sidecar)
        }
        Some(toml::Value::String(value)) => {
            anyhow::bail!(
                "doctrine.toml: unknown `[design] {MAP_DELIVERY_KEY}` value \"{value}\" — {}",
                accepted()
            )
        }
        Some(other) => anyhow::bail!(
            "doctrine.toml: `[design] {MAP_DELIVERY_KEY}` must be a string, not {} — {}",
            other.type_str(),
            accepted()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `[design]` entry of a `doctrine.toml` body, raw.
    fn design(body: &str) -> Option<toml::Value> {
        let doc: toml::Table = toml::from_str(body).expect("the fixture is well-formed TOML");
        doc.get("design").cloned()
    }

    fn resolve(body: &str) -> anyhow::Result<MapDelivery> {
        resolve_map_delivery(design(body).as_ref())
    }

    #[test]
    fn resolve_map_delivery_accepts_the_two_modes_and_defaults_to_relay() {
        let sidecar = format!("[design]\n{MAP_DELIVERY_KEY} = \"{MAP_DELIVERY_SIDECAR}\"\n");
        let relay = format!("[design]\n{MAP_DELIVERY_KEY} = \"{MAP_DELIVERY_RELAY}\"\n");
        for (body, expected) in [
            ("", MapDelivery::Relay),
            ("[design]\n", MapDelivery::Relay),
            (relay.as_str(), MapDelivery::Relay),
            (sidecar.as_str(), MapDelivery::Sidecar),
        ] {
            assert_eq!(resolve(body).expect("accepted"), expected, "{body:?}");
        }
    }

    #[test]
    fn resolve_map_delivery_refuses_each_malformation_naming_the_accepted_form() {
        for (body, names) in [
            ("design = 1\n", "not a table"),
            (
                "[design]\nmap_delivry = \"relay\"\n",
                "unknown key `[design] map_delivry`",
            ),
            (
                "[design]\nmap_delivery = 42\n",
                "must be a string, not integer",
            ),
            ("[design]\nmap_delivery = \"tree\"\n", "value \"tree\""),
        ] {
            let refused = resolve(body).expect_err(body).to_string();
            assert!(refused.contains(names), "{body:?} → {refused}");
            assert!(
                refused.contains(&format!(
                    "{MAP_DELIVERY_KEY} = \"{MAP_DELIVERY_RELAY}\" | \"{MAP_DELIVERY_SIDECAR}\""
                )),
                "{body:?} names the accepted form: {refused}"
            );
        }
    }
}
