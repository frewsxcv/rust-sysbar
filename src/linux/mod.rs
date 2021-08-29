use std::sync::Arc;

struct MyTray {
    #[allow(clippy::type_complexity)]
    menu: Vec<(String, Arc<Box<dyn Fn()>>)>,
}

impl ksni::Tray for MyTray {
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let mut menu_items = Vec::new();
        for entry in self.menu.clone().into_iter() {
            let item = ksni::MenuItem::Standard(ksni::menu::StandardItem {
                label: entry.0.to_owned(),
                activate: Box::new(move |_| {
                    entry.1();
                }),
                ..Default::default()
            });
            menu_items.push(item);
        }
        menu_items
    }
}

unsafe impl Send for MyTray {}

pub struct LinuxSysbar {
    tray_handle: ksni::Handle<MyTray>,
}

impl LinuxSysbar {
    pub fn new(_name: &str) -> Self {
        let tray = ksni::TrayService::new(MyTray { menu: Vec::new() });
        let tray_handle = tray.handle();
        tray.spawn();
        Self { tray_handle }
    }

    pub fn add_item(&mut self, label: &str, cbs: Box<dyn Fn()>) {
        let cbs = Arc::new(cbs);
        self.tray_handle.update(move |tray| {
            tray.menu.push((label.to_owned(), cbs.clone()));
        });
    }

    pub fn add_quit_item(&mut self, label: &str) {
        self.add_item(
            label,
            Box::new(|| {
                std::process::exit(0);
            }),
        );
    }

    pub fn display(&mut self) {
        loop {
            std::thread::park();
        }
    }
}
