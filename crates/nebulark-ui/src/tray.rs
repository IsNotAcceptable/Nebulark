use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder,
};

pub struct NebularkTray {
    _tray: TrayIcon,
    pub open_item_id: tray_icon::menu::MenuId,
    pub connect_item_id: tray_icon::menu::MenuId,
    pub disconnect_item_id: tray_icon::menu::MenuId,
    pub quit_item_id: tray_icon::menu::MenuId,
}

impl NebularkTray {
    pub fn new() -> anyhow::Result<Self> {
        let icon = load_icon()?;
        let open_item = MenuItem::new("Open Nebulark", true, None);
        let connect_item = MenuItem::new("Connect", true, None);
        let disconnect_item = MenuItem::new("Disconnect", false, None);
        let quit_item = MenuItem::new("Quit", true, None);

        let open_item_id = open_item.id().clone();
        let connect_item_id = connect_item.id().clone();
        let disconnect_item_id = disconnect_item.id().clone();
        let quit_item_id = quit_item.id().clone();

        let menu = Menu::new();
        menu.append(&open_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&connect_item)?;
        menu.append(&disconnect_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit_item)?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Nebulark — Disconnected")
            .with_icon(icon)
            .build()?;

        Ok(Self {
            _tray: tray,
            open_item_id,
            connect_item_id,
            disconnect_item_id,
            quit_item_id,
        })
    }

    pub fn set_connected(&mut self, connected: bool) {
        let tooltip = if connected {
            "Nebulark — Connected"
        } else {
            "Nebulark — Disconnected"
        };
        self._tray.set_tooltip(Some(tooltip)).ok();
        self._tray
            .set_icon(Some(
                load_icon().unwrap_or_else(|_| load_icon().unwrap()),
            ))
            .ok();
        let _ = connected;
    }
}

fn load_icon() -> anyhow::Result<tray_icon::Icon> {
    let bytes = include_bytes!("../assets/icons/nebulark-32.png");
    let img = image::load_from_memory(bytes)?.into_rgba8();
    let (w, h) = img.dimensions();
    Ok(tray_icon::Icon::from_rgba(img.into_raw(), w, h)?)
}