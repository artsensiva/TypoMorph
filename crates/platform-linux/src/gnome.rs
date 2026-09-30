use std::future::Future;
use std::time::Duration;

use crate::{LayoutSwitcher, PlatformError};

const DESTINATION: &str = "org.typomorph.Layout1";
const PATH: &str = "/org/typomorph/Layout1";
const INTERFACE: &str = "org.typomorph.Layout1";
const CALL_TIMEOUT: Duration = Duration::from_millis(250);

type StateReply = (u32, bool, String, String, Vec<String>);

pub struct GnomeShellSwitcher {
    connection: zbus::Connection,
}

fn unavailable(message: &str) -> PlatformError {
    PlatformError::UnsupportedBackend(message.to_owned())
}

fn bounded<T>(future: impl Future<Output = zbus::Result<T>>) -> Result<T, PlatformError> {
    async_io::block_on(futures_lite::future::or(
        async {
            future.await.map_err(|_| unavailable(
                "GNOME Layout Bridge is unavailable; enable the compatible TypoMorph Shell integration",
            ))
        },
        async {
            async_io::Timer::after(CALL_TIMEOUT).await;
            Err(unavailable(
                "GNOME Layout Bridge timed out; no text replacement is permitted",
            ))
        },
    ))
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_+-".contains(&c))
}

fn validate_state(state: StateReply) -> Result<String, PlatformError> {
    let (version, ready, source_type, current, layouts) = state;
    if version != 1
        || !ready
        || source_type != "xkb"
        || !valid_id(&current)
        || layouts.len() > 128
        || !layouts.iter().all(|id| valid_id(id))
        || !layouts.contains(&current)
        || layouts
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            != layouts.len()
    {
        return Err(unavailable(
            "GNOME Layout Bridge did not report an eligible XKB source",
        ));
    }
    Ok(current)
}

fn validate_confirmation(reply: (bool, String, String), target: &str) -> Result<(), PlatformError> {
    if reply.0 && reply.1 == "xkb" && reply.2 == target {
        Ok(())
    } else {
        Err(unavailable(
            "GNOME Layout Bridge did not confirm the requested layout",
        ))
    }
}

impl GnomeShellSwitcher {
    pub fn connect() -> Result<Self, PlatformError> {
        let switcher = Self {
            connection: bounded(zbus::Connection::session())?,
        };
        switcher.current_layout()?;
        Ok(switcher)
    }

    pub fn current_layout(&self) -> Result<String, PlatformError> {
        let reply = bounded(self.connection.call_method(
            Some(DESTINATION),
            PATH,
            Some(INTERFACE),
            "GetState",
            &(),
        ))?;
        let state: StateReply = reply
            .body()
            .deserialize()
            .map_err(|_| unavailable("GNOME Layout Bridge returned an invalid state reply"))?;
        validate_state(state)
    }
}

impl LayoutSwitcher for GnomeShellSwitcher {
    fn switch_from_to(&self, expected: &str, target: &str) -> Result<(), PlatformError> {
        if !valid_id(expected) || !valid_id(target) {
            return Err(unavailable("invalid layout identifier"));
        }
        let reply = bounded(self.connection.call_method(
            Some(DESTINATION),
            PATH,
            Some(INTERFACE),
            "Switch",
            &(expected, target),
        ))?;
        let confirmation: (bool, String, String) = reply
            .body()
            .deserialize()
            .map_err(|_| unavailable("GNOME Layout Bridge returned an invalid switch reply"))?;
        validate_confirmation(confirmation, target)?;
        // Re-read state to catch changes between activation and the reply.
        if self.current_layout()? != target {
            return Err(unavailable("GNOME layout changed before replacement"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> StateReply {
        (
            1,
            true,
            "xkb".into(),
            "us".into(),
            vec!["us".into(), "ru".into()],
        )
    }

    #[test]
    fn requires_ready_matching_protocol_and_real_xkb_source() {
        assert_eq!(validate_state(state()).unwrap(), "us");
        let mut cases = Vec::new();
        let mut s = state();
        s.0 = 2;
        cases.push(s);
        let mut s = state();
        s.1 = false;
        cases.push(s);
        let mut s = state();
        s.2 = "ibus".into();
        cases.push(s);
        let mut s = state();
        s.3 = "de".into();
        cases.push(s);
        let mut s = state();
        s.4.push("us".into());
        cases.push(s);
        for s in cases {
            assert!(validate_state(s).is_err());
        }
    }

    #[test]
    fn switch_requires_positive_exact_xkb_confirmation() {
        for reply in [
            (false, "xkb".into(), "ru".into()),
            (true, "ibus".into(), "ru".into()),
            (true, "xkb".into(), "us".into()),
            (true, "".into(), "".into()),
        ] {
            assert!(validate_confirmation(reply, "ru").is_err());
        }
        assert!(validate_confirmation((true, "xkb".into(), "ru".into()), "ru").is_ok());
    }

    #[test]
    fn identifiers_are_bounded_and_content_free() {
        for id in ["", "ru\n", "ru';", "русский"] {
            assert!(!valid_id(id));
        }
        assert!(!valid_id(&"a".repeat(129)));
        for id in ["us", "ru", "us+intl", "test_layout-1"] {
            assert!(valid_id(id));
        }
    }

    #[test]
    fn nonresponding_transport_has_a_deadline() {
        let result: Result<(), PlatformError> = bounded(std::future::pending());
        assert!(
            matches!(result, Err(PlatformError::UnsupportedBackend(message)) if message.contains("timed out"))
        );
    }
}
