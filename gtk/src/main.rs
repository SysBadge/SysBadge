use std::time::Duration;

use gtk::prelude::*;
use relm4::{
    component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender},
    gtk,
    loading_widgets::LoadingWidgets,
    view, RelmApp, RelmWidgetExt,
};

#[tracker::track]
struct App {
    #[no_eq]
    badge: Option<u8>,
    counter: u8,
}

#[derive(Debug)]
enum Msg {
    Increment,
    Decrement,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = u8;
    type Input = Msg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::ApplicationWindow {
            if model.badge.is_some() {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                    set_margin_all: 5,

                    gtk::Button {
                        set_label: "Increment",
                        connect_clicked => Msg::Increment,
                    },

                    gtk::Button {
                        set_label: "Decrement",
                        connect_clicked => Msg::Decrement,
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &format!("Counter: {}", model.counter),
                        set_margin_all: 5,
                    }
                }
            } else {
                gtk::Box {
                    gtk::Label {
                        set_label: "Please connect Sysbadge...",
                        set_margin_all: 5,
                    },

                    gtk::Spinner {
                        start: (),
                        set_halign: gtk::Align::Center,
                    }
                }
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
        tokio::time::sleep(Duration::from_secs(1)).await;

        let model = App {
            badge: None,
            counter,
            tracker: 0,
        };

        std::thread::spawn(|| sender.send);

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
            Msg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            Msg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
    }
}

fn main() {
    pretty_env_logger::init();

    let app = RelmApp::new("eu.kloenk.sysbadge.gtk");
    app.run_async::<App>(0);
}
