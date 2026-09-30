use platform_linux::InputContext;

#[test]
#[ignore = "requires integrations/gnome/tests/run_context_test.py"]
fn metadata_probe_rejects_password_selection_unknown_and_invalid_positions() {
    assert_eq!(
        std::env::var("TYPOMORPH_PRIVATE_CONTEXT_TEST").as_deref(),
        Ok("1")
    );
    let control = zbus::blocking::Connection::session().unwrap();
    let context = InputContext::connect().expect("private metadata bus");
    let snapshot = context.snapshot().expect("eligible synthetic field");
    assert_eq!(snapshot.caret, 3);
    assert_eq!(snapshot.characters, 10);
    assert_eq!(snapshot.field, "/entry");
    for mode in 1u32..=16 {
        control
            .call_method(
                Some("org.a11y.atspi.Registry"),
                "/fixture",
                Some("org.typomorph.ContextFixture"),
                "SetMode",
                &(mode,),
            )
            .unwrap();
        if mode == 8 {
            assert!(context.check().is_ok(), "GTK 4 may omit ENABLED");
            continue;
        }
        let expected = match mode {
            1 => "focused_object_ineligible",
            2 | 4 => "no_eligible_focus",
            3 => "selection_present",
            5 => "invalid_position",
            6 => "field_changed_or_recheck_unavailable",
            7 => "object_role_unavailable",
            9..=15 => "focused_object_ineligible",
            16 => "field_changed_or_recheck_unavailable",
            _ => unreachable!(),
        };
        assert_eq!(context.check().err(), Some(expected), "mode {mode}");
        if mode == 1 || mode == 6 || (9..=15).contains(&mode) {
            let reply = control
                .call_method(
                    Some("org.a11y.atspi.Registry"),
                    "/fixture",
                    Some("org.typomorph.ContextFixture"),
                    "TextQueries",
                    &(),
                )
                .unwrap();
            assert_eq!(
                reply.body().deserialize::<u32>().unwrap(),
                0,
                "password fields must be rejected before any text-interface query"
            );
        }
    }
}
