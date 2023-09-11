mod member_list;

use druid::widget::{Container, DisabledIf, Flex, Label, LensWrap, Split, Tabs};
use druid::{
    AppDelegate, AppLauncher, BoxConstraints, Command, Data, DelegateCtx, Env, Event, EventCtx,
    Handled, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx, Selector, Size, Target, UpdateCtx,
    Widget, WidgetExt, WindowDesc,
};
use std::sync::{Arc, Mutex};
use sysbadge_usb::rusb::{Context, Device, Hotplug, HotplugBuilder, UsbContext};
use sysbadge_usb::UsbSysbadge;

const BADGE_ARRIVED: Selector<Device<Context>> = Selector::new("eu.kloenk.sysbadge.badge_arrived");
const BADGE_LEFT: Selector<Device<Context>> = Selector::new("eu.kloenk.sysbadge.badge_left");

#[derive(Clone, Data, Lens, Debug)]
struct State {
    badge: Option<Arc<UsbSysbadge>>,
    names: druid::im::Vector<String>,
}

impl State {
    fn new() -> Self {
        Self {
            badge: None,
            names: druid::im::Vector::new(),
        }
    }
}

fn build_ui() -> impl Widget<State> {
    let member_list = member_list::build_member_list();

    Tabs::new()
        .with_tab("Member List", member_list)
        .with_tab("Foo", Label::new("Eth"))
        .with_tab("Bar", Label::new("IPFS"))
        .on_added(|s, l, t, e| {
            let ext = l.get_external_handle();

            let ctx = Context::new().unwrap();
            let mut reg = HotplugBuilder::new()
                .vendor_id(sysbadge_usb::VID)
                .product_id(sysbadge_usb::PID)
                .enumerate(true)
                .register(ctx.clone(), Box::new(HotplugController { sink: ext }))
                .unwrap();

            std::thread::spawn(move || {
                let reg = reg;
                loop {
                    ctx.handle_events(None).unwrap();
                }
                let _ = reg;
            });
        })
}

fn main() {
    let main_window = WindowDesc::new(build_ui())
        .window_size((600.0, 400.0))
        .title("SysBadge");
    let initial_data = State::new();

    AppLauncher::with_window(main_window)
        .delegate(Deletage {})
        .log_to_console()
        .launch(initial_data)
        .expect("Failed to launch application");
}

struct Deletage;

impl AppDelegate<State> for Deletage {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        target: Target,
        cmd: &Command,
        data: &mut State,
        env: &Env,
    ) -> Handled {
        if let Some(badge) = cmd.get(BADGE_ARRIVED) {
            //data.badge = Somebadge);
            println!("Badge arrived: {:?}", badge);
            let badge = UsbSysbadge::open(badge.open().unwrap()).unwrap();

            for x in 0..badge.member_count().unwrap() {
                let member = badge.member_name(x).unwrap();
                data.names.push_back(member);
            }

            data.badge = Some(Arc::new(badge));
            return Handled::Yes;
        }
        if let Some(badge) = cmd.get(BADGE_LEFT) {
            println!("Badge left: {:?}", badge);
            data.badge = None;
            data.names.clear();
            return Handled::Yes;
        }
        Handled::No
    }
}

pub struct HotplugController {
    sink: druid::ExtEventSink,
}

impl Hotplug<Context> for HotplugController {
    fn device_arrived(&mut self, device: Device<Context>) {
        self.sink
            .submit_command(BADGE_ARRIVED, device, Target::Global)
            .unwrap();
    }

    fn device_left(&mut self, device: Device<Context>) {
        self.sink
            .submit_command(BADGE_LEFT, device, Target::Global)
            .unwrap();
    }
}
