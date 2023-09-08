use gtk::prelude::{ButtonExt, ToggleButtonExt, WidgetExt};
use relm4::component::{AsyncComponentParts, SimpleAsyncComponent};
use relm4::*;

pub(crate) struct HeaderModel;

#[derive(Debug)]
pub(crate) enum HeaderOutput {
    View,
    Edit,
    Export,
}

#[relm4::component(pub async)]
impl SimpleAsyncComponent for HeaderModel {
    type Init = ();
    type Input = ();
    type Output = HeaderOutput;

    view! {
        #[root]
        gtk::HeaderBar {
            #[wrap(Some)]
            set_title_widget = &gtk::Box {
                add_css_class: "linked",
                #[name = "group"]
                gtk::ToggleButton {
                    set_label: "View",
                    set_active: true,
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(HeaderOutput::View).unwrap()
                        }
                    },
                },
                gtk::ToggleButton {
                    set_label: "Edit",
                    set_group: Some(&group),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(HeaderOutput::Edit).unwrap()
                        }
                    },
                },
                gtk::ToggleButton {
                    set_label: "Export",
                    set_group: Some(&group),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(HeaderOutput::Export).unwrap()
                        }
                    },
                },
            }
        }
    }

    async fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = HeaderModel;
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }
}
