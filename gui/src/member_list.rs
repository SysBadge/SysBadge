use crate::State;
use druid::widget::{Flex, Label, LensWrap, List, Scroll};
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx, Widget, WidgetExt,
};
use std::sync::Arc;
use sysbadge_usb::UsbSysbadge;

pub(crate) fn build_member_list() -> impl Widget<State> {
    let name = LensWrap::new(
        Label::dynamic(|badge: &Option<Arc<UsbSysbadge>>, _| {
            if let Some(badge) = badge.as_ref() {
                format!(
                    "Name: {}",
                    badge
                        .system_name()
                        .unwrap_or_else(|_| "Unknown".to_string())
                )
            } else {
                "No badge connected".to_string()
            }
        }),
        State::badge,
    );

    let list = List::new(|| Label::dynamic(|name: &String, _| format!("Name: {}", name)))
        .lens(State::names);

    Flex::column()
        .with_child(name)
        .with_default_spacer()
        .with_flex_child(Scroll::new(list).vertical().content_must_fill(true), 1.0)
}
