//! Bounded AT-SPI metadata probe. Never requests text, names, descriptions or cache data.
use core_engine::context::FieldSnapshot;
use serde::{de::DeserializeOwned, Serialize};
use std::cell::Cell;
use std::collections::{HashSet, VecDeque};
use std::time::Duration;
use zbus::zvariant::{DynamicType, OwnedObjectPath, OwnedValue, Type};

type Object = (String, OwnedObjectPath);
const ACCESSIBLE: &str = "org.a11y.atspi.Accessible";
const TEXT: &str = "org.a11y.atspi.Text";
const BUDGET: Duration = Duration::from_millis(500);
const MAX_NODES: usize = 256;

pub struct InputContext {
    connection: zbus::Connection,
}

async fn call<R: DeserializeOwned + Type, B: Serialize + DynamicType>(
    connection: &zbus::Connection,
    object: &Object,
    interface: &str,
    method: &str,
    body: &B,
) -> Option<R> {
    connection
        .call_method(
            Some(object.0.as_str()),
            object.1.as_str(),
            Some(interface),
            method,
            body,
        )
        .await
        .ok()?
        .body()
        .deserialize()
        .ok()
}

fn bounded<T>(future: impl std::future::Future<Output = Option<T>>) -> Option<T> {
    async_io::block_on(futures_lite::future::or(future, async {
        async_io::Timer::after(BUDGET).await;
        None
    }))
}

fn has(states: &[u32], bit: usize) -> bool {
    states
        .get(bit / 32)
        .is_some_and(|word| word & (1 << (bit % 32)) != 0)
}

fn editable(role: u32, states: &[u32]) -> bool {
    // Password (40), terminal (60), unknown and other roles never qualify.
    matches!(role, 61 | 79)
        && !has(states, 6)
        && !has(states, 43) // READ_ONLY overrides even contradictory EDITABLE metadata.
        && [7, 12, 24, 25, 30].iter().all(|bit| has(states, *bit))
}

impl InputContext {
    pub fn connect() -> Option<Self> {
        bounded(async {
            let session = zbus::Connection::session().await.ok()?;
            let address: String = session
                .call_method(
                    Some("org.a11y.Bus"),
                    "/org/a11y/bus",
                    Some("org.a11y.Bus"),
                    "GetAddress",
                    &(),
                )
                .await
                .ok()?
                .body()
                .deserialize()
                .ok()?;
            let connection = zbus::connection::Builder::address(address.as_str())
                .ok()?
                .build()
                .await
                .ok()?;
            Some(Self { connection })
        })
    }

    pub fn snapshot(&self) -> Option<FieldSnapshot> {
        self.check().ok()
    }

    /// Fixed diagnostic codes only: no object identity or input metadata is exposed.
    pub fn check(&self) -> Result<FieldSnapshot, &'static str> {
        let stage = Cell::new("registry_unavailable");
        bounded(async { Some(self.discover(&stage).await.ok_or(stage.get())) })
            .unwrap_or(Err("metadata_timeout"))
    }

    async fn discover(&self, stage: &Cell<&'static str>) -> Option<FieldSnapshot> {
        let root = (
            "org.a11y.atspi.Registry".into(),
            OwnedObjectPath::try_from("/org/a11y/atspi/accessible/root").ok()?,
        );
        let apps: Vec<Object> =
            call(&self.connection, &root, ACCESSIBLE, "GetChildren", &()).await?;
        let mut queue: VecDeque<(Object, Option<Object>, usize)> =
            apps.into_iter().map(|app| (app, None, 0)).collect();
        let mut visited = HashSet::new();
        let mut found = None;
        while let Some((object, mut window, depth)) = queue.pop_front() {
            if visited.len() >= MAX_NODES || queue.len() >= MAX_NODES || depth > 32 {
                stage.set("traversal_limit");
                return None;
            }
            // AT-SPI's null sentinel denotes no accessible object, not an unknown field.
            if object.1.as_str() == "/org/a11y/atspi/null" {
                continue;
            }
            if !visited.insert(object.clone()) {
                stage.set("repeated_object");
                return None;
            }
            stage.set("object_role_unavailable");
            let role: u32 = call(&self.connection, &object, ACCESSIBLE, "GetRole", &()).await?;
            stage.set("object_state_unavailable");
            let states: Vec<u32> =
                call(&self.connection, &object, ACCESSIBLE, "GetState", &()).await?;
            if has(&states, 6) {
                continue;
            }
            if matches!(role, 16 | 23 | 69) {
                if !has(&states, 1) {
                    continue;
                }
                window = Some(object.clone());
            }
            if has(&states, 12) {
                stage.set("focused_object_without_active_window");
                let window = window.as_ref()?;
                if !editable(role, &states) || !self.enabled_compatible(&object, &states).await {
                    stage.set("focused_object_ineligible");
                    return None;
                }
                if found.is_some() {
                    stage.set("ambiguous_focus");
                    return None;
                }
                stage.set("text_interface_unavailable");
                let interfaces: Vec<String> =
                    call(&self.connection, &object, ACCESSIBLE, "GetInterfaces", &()).await?;
                if !interfaces.iter().any(|i| i == TEXT) {
                    return None;
                }
                stage.set("selection_metadata_unavailable");
                let selections: i32 =
                    call(&self.connection, &object, TEXT, "GetNSelections", &()).await?;
                if selections != 0 {
                    stage.set("selection_present");
                    return None;
                }
                stage.set("position_metadata_unavailable");
                let caret = self.number(&object, "CaretOffset").await?;
                let characters = self.number(&object, "CharacterCount").await?;
                if caret < 0 || characters < caret {
                    stage.set("invalid_position");
                    return None;
                }
                stage.set("field_changed_or_recheck_unavailable");
                // Catch state changes during the metadata reads. Not an atomic snapshot.
                let again: Vec<u32> =
                    call(&self.connection, &object, ACCESSIBLE, "GetState", &()).await?;
                let role_again: u32 =
                    call(&self.connection, &object, ACCESSIBLE, "GetRole", &()).await?;
                if role_again != role
                    || !editable(role_again, &again)
                    || !self.enabled_compatible(&object, &again).await
                {
                    return None;
                }
                let selections_again: i32 =
                    call(&self.connection, &object, TEXT, "GetNSelections", &()).await?;
                if role_again != role
                    || !editable(role_again, &again)
                    || selections_again != 0
                    || self.number(&object, "CaretOffset").await? != caret
                    || self.number(&object, "CharacterCount").await? != characters
                {
                    return None;
                }
                found = Some((
                    FieldSnapshot {
                        application: object.0.clone(),
                        window: format!("{}:{}", window.0, window.1),
                        field: object.1.to_string(),
                        caret,
                        characters,
                    },
                    window.clone(),
                    object.clone(),
                ));
                continue;
            }
            stage.set("children_unavailable");
            let children: Vec<Object> =
                call(&self.connection, &object, ACCESSIBLE, "GetChildren", &()).await?;
            queue.extend(
                children
                    .into_iter()
                    .map(|child| (child, window.clone(), depth + 1)),
            );
        }
        stage.set("no_eligible_focus");
        let (snapshot, window, object) = found?;
        stage.set("final_recheck_failed");
        let window_states: Vec<u32> =
            call(&self.connection, &window, ACCESSIBLE, "GetState", &()).await?;
        let field_states: Vec<u32> =
            call(&self.connection, &object, ACCESSIBLE, "GetState", &()).await?;
        let role: u32 = call(&self.connection, &object, ACCESSIBLE, "GetRole", &()).await?;
        if !has(&window_states, 1)
            || !editable(role, &field_states)
            || !self.enabled_compatible(&object, &field_states).await
        {
            return None;
        }
        let selections: i32 = call(&self.connection, &object, TEXT, "GetNSelections", &()).await?;
        if !has(&window_states, 1)
            || !editable(role, &field_states)
            || selections != 0
            || self.number(&object, "CaretOffset").await? != snapshot.caret
            || self.number(&object, "CharacterCount").await? != snapshot.characters
        {
            return None;
        }
        Some(snapshot)
    }

    async fn enabled_compatible(&self, object: &Object, states: &[u32]) -> bool {
        if has(states, 8) {
            return true;
        }
        // GTK 4 collect_states exports SENSITIVE for enabled widgets, but not ENABLED.
        // Identify the owning toolkit via individual non-content properties, never GetAttributes.
        let Some(app): Option<Object> =
            call(&self.connection, object, ACCESSIBLE, "GetApplication", &()).await
        else {
            return false;
        };
        if app.0 != object.0 || app.1.as_str() == "/org/a11y/atspi/null" {
            return false;
        }
        let Some(name) = self.toolkit_property(&app, "ToolkitName").await else {
            return false;
        };
        if name != "GTK" {
            return false;
        }
        let Some(version) = self.toolkit_property(&app, "Version").await else {
            return false;
        };
        let parts: Vec<_> = version.split('.').collect();
        version.len() <= 32
            && parts.len() == 3
            && parts[0] == "4"
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
    }

    async fn toolkit_property(&self, app: &Object, property: &str) -> Option<String> {
        let value: OwnedValue = call(
            &self.connection,
            app,
            "org.freedesktop.DBus.Properties",
            "Get",
            &("org.a11y.atspi.Application", property),
        )
        .await?;
        String::try_from(value).ok()
    }

    async fn number(&self, object: &Object, property: &str) -> Option<i32> {
        let value: OwnedValue = call(
            &self.connection,
            object,
            "org.freedesktop.DBus.Properties",
            "Get",
            &(TEXT, property),
        )
        .await?;
        i32::try_from(value).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn states() -> Vec<u32> {
        vec![
            [7, 8, 12, 24, 25, 30]
                .iter()
                .fold(0, |bits, bit| bits | (1 << bit)),
            0,
        ]
    }
    #[test]
    fn only_focused_editable_plain_text_roles_qualify() {
        assert!(editable(61, &states()));
        assert!(editable(79, &states()));
        for role in [0, 40, 60, 67] {
            assert!(!editable(role, &states()));
        }
        for bit in [7, 12, 24, 25, 30] {
            let mut s = states();
            s[0] &= !(1 << bit);
            assert!(!editable(61, &s));
        }
        let mut s = states();
        s[0] |= 1 << 6;
        assert!(!editable(61, &s));
        assert!(!editable(61, &[]));
        let mut readonly = states();
        readonly[1] |= 1 << (43 - 32);
        assert!(!editable(61, &readonly));
    }
    #[test]
    fn a_hung_accessibility_provider_cannot_wait_forever() {
        assert!(bounded::<()>(std::future::pending()).is_none());
    }
}
