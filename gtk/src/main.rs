mod components;
mod usb;

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::time::Duration;

use crate::components::{HeaderInput, HeaderModel, HeaderOutput};
use anyhow::{Context, Result};
use gtk::prelude::*;
use log::*;
use relm4::component::AsyncController;
use relm4::{
    component::{
        AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncComponentSender,
    },
    gtk,
    loading_widgets::LoadingWidgets,
    view, Controller, RelmApp, RelmWidgetExt,
};
use sysbadge_usb::rusb::{DeviceHandle, UsbContext};
use sysbadge_usb::{rusb, UsbSysbadge};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum AppMode {
    View,
    Edit,
    Export,
}

pub(crate) struct App {
    badges: HashMap<String, UsbSysbadge<rusb::Context>>,
    header: AsyncController<HeaderModel>,
    tab: AppMode,
}

enum Msg {
    AddBadge(DeviceHandle<rusb::Context>),
    RemoveBadge(rusb::Device<rusb::Context>),
    SetMode(AppMode),
}

impl Debug for Msg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Msg::AddBadge(_) => write!(f, "AddBadge"),
            Msg::RemoveBadge(_) => write!(f, "RemoveBadge"),
            Msg::SetMode(m) => write!(f, "SetMode: {:?}", m),
        }
    }
}

impl App {
    fn add_badge(&mut self, handle: DeviceHandle<rusb::Context>) -> Result<()> {
        let mut badge = UsbSysbadge::open(handle)?;

        let name = badge.system_name()?;
        let name = format!(
            "{} ({}:{})",
            name,
            badge.handle().device().bus_number(),
            badge.handle().device().address()
        );
        info!("Got new badge for {}", name);
        if let Err(e) = self.header.sender().send(HeaderInput::Add(name.clone())) {
            warn!("Failed to send header input: {:?}", e);
        }

        self.badges.insert(name, badge);

        Ok(())
    }

    fn remove_badge(&mut self, handle: rusb::Device<rusb::Context>) -> Result<()> {
        self.badges
            .retain(|_, badge| badge.handle().device().address() != handle.address());
        info!("Removed badge");
        trace!("Badges: {:?}", self.badges.keys());
        Ok(())
    }
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = ();
    type Input = Msg;
    type Output = ();
    type CommandOutput = ();

    view! {
        main_window = gtk::Window {
            set_default_width: 500,
            set_default_height: 250,
            set_titlebar: Some(model.header.widget()),

            gtk::CenterBox {
                set_hexpand: true,
                set_vexpand: true,

                #[wrap(Some)]
                set_center_widget = match *(&model.tab) {
                    AppMode::View => gtk::Label {
                        set_label: "View",
                    },
                    _ => gtk::Label {
                        set_label: "Placeholder",
                    }
                },
            },

            connect_close_request[sender] => move |_| {
                // TODO: sender.input(AppMsg::CloseRequest);
                gtk::Inhibit(false)
            }
        }
    }

    fn init_loading_widgets(root: &mut Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local_ref]
            root {
                set_title: Some("Simple app"),
                set_default_size: (300, 100),

                // This will be removed automatically by
                // LoadingWidgets when the full view has loaded
                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_halign: gtk::Align::Center,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    async fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let header: AsyncController<HeaderModel> =
            HeaderModel::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    HeaderOutput::View => Msg::SetMode(AppMode::View),
                    HeaderOutput::Edit => Msg::SetMode(AppMode::Edit),
                    HeaderOutput::Export => Msg::SetMode(AppMode::Export),
                });

        let model = App {
            badges: HashMap::new(),
            tab: AppMode::View,
            header,
        };

        let usb_sender = sender.clone();
        std::thread::spawn(move || {
            usb::run(usb_sender);
        });

        let state = &model.tab;
        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            Msg::AddBadge(handle) => {
                if let Err(e) = self.add_badge(handle) {
                    error!("Failed to add badge: {}", e);
                }
            }
            Msg::RemoveBadge(handle) => {
                if let Err(e) = self.remove_badge(handle) {
                    error!("Failed to remove badge: {}", e);
                }
            }
            Msg::SetMode(m) => {
                self.tab = m;
                info!("Set mode to {:?}", self.tab);
            }
        }
    }
}

fn main() {
    pretty_env_logger::init();

    let app = RelmApp::new("eu.kloenk.sysbadge.gtk");
    app.run_async::<App>(());
}
