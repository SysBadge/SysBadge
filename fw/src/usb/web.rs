use crate::RpFlashMutex;
use alloc::borrow::ToOwned;
use alloc::string::String;
use defmt::*;
use embassy_net::Stack;
use embassy_rp::pac::xip_ctrl::regs::Stat;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_usb::class::cdc_ncm::embassy_net::Device;
use embedded_io_async::Write;
use picoserve::extract::State;
use picoserve::request::Request;
use picoserve::response::IntoResponse;
use picoserve::routing::{get, parse_path_segment, post};
use picoserve::Router;
use static_cell::make_static;
use sysbadge::badge::{CurrentMembers, CurrentMenu, MemberCell};
use sysbadge::system::Member;
use sysbadge::System;

struct EmbassyTimer;

impl picoserve::Timer for EmbassyTimer {
    type Duration = embassy_time::Duration;
    type TimeoutError = embassy_time::TimeoutError;

    async fn run_with_timeout<F: core::future::Future>(
        &mut self,
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, Self::TimeoutError> {
        embassy_time::with_timeout(duration, future).await
    }
}

#[derive(Clone)]
struct WebState {
    badge: &'static Mutex<CriticalSectionRawMutex, crate::SysbadgeUc8151<'static>>,
    flash: &'static RpFlashMutex<'static>,
}

impl picoserve::extract::FromRef<WebState>
    for &'static Mutex<CriticalSectionRawMutex, crate::SysbadgeUc8151<'static>>
{
    fn from_ref(state: &WebState) -> Self {
        state.badge
    }
}

enum BadgeState {
    Locked,
    Name,
    Version,
    Member(u8, [u16; 4]),
}

impl core::str::FromStr for BadgeState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "locked" => Ok(Self::Locked),
            "name" => Ok(Self::Name),
            "version" => Ok(Self::Version),
            s if s.starts_with("member") => {
                let mut iter = s.split('-');
                iter.next();
                let mut len = 0;
                let mut arr = [0; 4];
                for x in iter {
                    // TODO: unwrap
                    arr[len as usize] = x.parse().map_err(|_| ())?;
                    len += 1;
                    if len > 4 {
                        return Err(());
                    }
                }
                Ok(Self::Member(len, arr))
            }
            _ => Err(()),
        }
    }
}

struct DataFile {}

impl picoserve::extract::FromRequest<WebState> for DataFile {
    type Rejection = ();

    async fn from_request(
        state: &WebState,
        request: &Request<'_>,
    ) -> Result<Self, Self::Rejection> {
        defmt::info!("len: {}", request.body().len());
        defmt::todo!()
    }
}

#[embassy_executor::task]
pub(crate) async fn web_server_task(
    stack: &'static Stack<Device<'static, { super::MTU }>>,
    badge: &'static Mutex<CriticalSectionRawMutex, crate::SysbadgeUc8151<'static>>,
    flash: &'static RpFlashMutex<'static>,
) {
    /*let app = make_static!(picoserve::Router::new()
    .route("/", get(get_counter)));*/
    type WebRouter = impl picoserve::routing::PathRouter<WebState> + 'static;
    let app: &Router<WebRouter, WebState> = make_static!(picoserve::Router::new()
        .route("/api/sysinfo", get(get_system_info))
        .route("/api/version", get(get_version))
        .route(("/api/members", parse_path_segment()), get(get_member))
        .route(("/api/set", parse_path_segment()), post(set_state))
        .route("/api/update", post(update)));

    let config = make_static!(picoserve::Config {
        start_read_request_timeout: None,
        read_request_timeout: None,
    });

    let buf: &mut [u8] = make_static!([0; 4096]);

    let rx_buffer: &mut [u8] = make_static!([0; 4096]);
    let tx_buffer: &mut [u8] = make_static!([0; 4096]);

    loop {
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, rx_buffer, tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        info!("Listening on TCP:80...");
        if let Err(e) = socket.accept(80).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());

        let (socket_rx, socket_tx) = socket.split();

        match picoserve::serve_with_state(
            app,
            EmbassyTimer,
            config,
            &mut [0; 2048],
            socket_rx,
            socket_tx,
            &WebState { badge, flash },
        )
        .await
        {
            Ok(handled_requests_count) => {
                info!(
                    "{} requests handled from {:?}",
                    handled_requests_count,
                    socket.remote_endpoint()
                );
            }
            Err(err) => error!("Failed to serve"), //error!("{:?}", err),
        }
    }
}

/*#[embassy_executor::task]
async fn web_task(
    stack: &'static Stack<Device<'static, { super::MTU }>>,
    app: &'static picoserve::Router<WebRouter, WebState>,
    config: &'static picoserve::Config<Duration>,
    state: WebState,
) -> ! {

}*/

async fn get_system_info(State(state): State<WebState>) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct SystemInfo {
        name: String,
        hid: Option<String>,
        members: u32,
    }
    let state = state.badge.lock().await;

    let hid = state.system.reader().unwrap().which().unwrap();
    let hid = if let sysbadge::system::system_capnp::system::Which::PkHid(reader) = hid {
        reader.unwrap().to_string().ok()
    } else {
        None
    };

    picoserve::response::Json(SystemInfo {
        name: state.system.name().to_owned(),
        hid,
        members: state.system.member_count() as u32,
    })
}

async fn get_member(id: u32, State(state): State<WebState>) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct Member {
        id: u32,
        name: String,
        pronouns: String,
    }

    let state = state.badge.lock().await;
    let member = state.system.member(id as usize);
    let member = Member {
        id,
        name: member.name().to_owned(),
        pronouns: member.pronouns().to_owned(),
    };

    picoserve::response::Json(member)
}

async fn get_version(State(state): State<WebState>) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct Version {
        serial: [u8; 16],
        semver: &'static str,
    }

    picoserve::response::Json(Version {
        serial: {
            let mut buf = [0; 8];
            let mut flash = state.flash.lock().await;
            defmt::unwrap!(flash.blocking_unique_id(&mut buf));
            let mut out = [0; 16];
            defmt::unwrap!(
                hex::encode_to_slice(&buf, &mut out),
                "Failed to encode serial"
            );
            out
        },
        semver: sysbadge::VERSION,
    })
}

async fn set_state(state: BadgeState, State(web_state): State<WebState>) -> impl IntoResponse {
    let mut badge = web_state.badge.lock().await;

    match state {
        // TODO: Locked instead of InvalidSystem
        BadgeState::Locked => badge.set_current(CurrentMenu::InvalidSystem),
        BadgeState::Name => badge.set_current(CurrentMenu::SystemName),
        BadgeState::Version => badge.set_current(CurrentMenu::Version),
        BadgeState::Member(len, arr) => {
            let member_state = CurrentMembers {
                members: arr.map(|x| MemberCell { id: x }),
                sel: (0, Default::default()),
                len,
            };
            badge.set_current(CurrentMenu::Member(member_state))
        }
    }

    badge.draw().unwrap();
    badge.display.update().unwrap();
    picoserve::response::Json("ok")
}

async fn update(State(web_state): State<WebState>, data: DataFile) -> impl IntoResponse {
    let mut badge = web_state.badge.lock().await;

    badge.draw().unwrap();
    badge.display.update().unwrap();
    picoserve::response::Json("ok")
}
