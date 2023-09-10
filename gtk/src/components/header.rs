use gtk::prelude::{ButtonExt, ToggleButtonExt, WidgetExt};
use log::debug;
use relm4::component::{AsyncComponentParts, SimpleAsyncComponent};
use relm4::*;

pub(crate) struct HeaderModel {
    list: Vec<String>,
}

#[derive(Debug)]
pub(crate) enum HeaderInput {
    Add(String),
    Remove(String),
}

#[derive(Debug)]
pub(crate) enum HeaderOutput {
    View,
    Edit,
    Export,
}

#[relm4::component(pub async)]
impl SimpleAsyncComponent for HeaderModel {
    type Init = ();
    type Input = HeaderInput;
    type Output = HeaderOutput;

    view! {
        #[root]
        gtk::HeaderBar {
            set_show_title_buttons: true,
            #[wrap(Some)]
            set_title_widget = &gtk::Box {
                gtk::DropDown {
                    set_valign: gtk::Align::Start,
                    set_enable_search: false,

                    /*#[wrap(Some)]
                    #[name(list)]
                    set_list_factory = &gtk::SignalListItemFactory {
                        connect_setup[sender] => move |_, item| {
                            let label = gtk::Label::new(None);
                            item.set_child(Some(&label));
                        },
                    },*/

                },
                /*gtk::ListBox {
                    set_valign: gtk::Align::Start,
                    set_selection_mode: gtk::SelectionMode::Single,

                    gtk::ListBoxRow {
                        gtk::Label {
                            set_label: "System",
                            set_valign: gtk::Align::Start,
                        },
                        gtk::Label {
                            set_label: "System name",
                            set_valign: gtk::Align::Start,
                        }
                    },
                },*/

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
        let model = HeaderModel { list: Vec::new() };
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        debug!("Header input: {:?}", message);
        match message {
            HeaderInput::Add(s) => {
                self.list.push(s);

                //self.widgets.list.set_items(&self.list);
            }
            HeaderInput::Remove(s) => {
                self.list.retain(|x| x != &s);
                //self.widgets.list.set_items(&self.list);
            }
        }
    }
}
