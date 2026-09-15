use ksni::{menu::*, Icon, MenuItem, Tray, TrayService};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct TypoMorphTray {
    pub paused: Arc<AtomicBool>,
    pub is_pro: bool,
}

impl Tray for TypoMorphTray {
    fn id(&self) -> String {
        "typomorph".into()
    }

    fn title(&self) -> String {
        "TypoMorph".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        let size = 22;
        let mut argb = Vec::with_capacity(size * size * 4);
        let is_paused = self.paused.load(Ordering::Relaxed);

        for y in 0..size {
            for x in 0..size {
                let inside_t = ((4..=6).contains(&y) && (4..=17).contains(&x))
                    || ((7..=17).contains(&y) && (9..=12).contains(&x));
                if inside_t {
                    argb.extend_from_slice(&[255, 255, 255, 255]); // Белый текст
                } else if is_paused {
                    argb.extend_from_slice(&[255, 128, 128, 128]); // Серый фон (на паузе)
                } else {
                    argb.extend_from_slice(&[255, 33, 150, 243]); // Фирменный синий фон
                }
            }
        }

        vec![Icon {
            width: size as i32,
            height: size as i32,
            data: argb,
        }]
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let is_paused = self.paused.load(Ordering::Relaxed);
        let tier_label = if self.is_pro {
            "TypoMorph: Pro Tier"
        } else {
            "TypoMorph: Free Tier"
        };
        let state_label = if is_paused {
            "Status: Paused"
        } else {
            "Status: Active"
        };
        let toggle_label = if is_paused { "Resume" } else { "Pause" };

        let paused_clone = Arc::clone(&self.paused);

        vec![
            StandardItem {
                label: format!("{tier_label} ({state_label})"),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: toggle_label.into(),
                activate: Box::new(move |_| {
                    let prev = paused_clone.load(Ordering::Relaxed);
                    paused_clone.store(!prev, Ordering::Relaxed);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit TypoMorph".into(),
                activate: Box::new(|_| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn spawn_tray(is_pro: bool, paused: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let tray = TypoMorphTray { paused, is_pro };
        let service = TrayService::new(tray);
        let handle = service.handle();
        service.spawn();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            handle.update(|_| {});
        }
    });
}
