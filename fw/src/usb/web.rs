use defmt::*;
use embassy_net::Stack;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_usb::class::cdc_ncm::embassy_net::Device;
use embedded_io_async::Write;
use picoserve::extract::State;
use picoserve::response::IntoResponse;
use picoserve::routing::get;
use picoserve::Router;
use static_cell::make_static;
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
}

impl picoserve::extract::FromRef<WebState>
    for &'static Mutex<CriticalSectionRawMutex, crate::SysbadgeUc8151<'static>>
{
    fn from_ref(state: &WebState) -> Self {
        state.badge
    }
}

#[embassy_executor::task]
pub(crate) async fn web_server_task(
    stack: &'static Stack<Device<'static, { super::MTU }>>,
    badge: &'static Mutex<CriticalSectionRawMutex, crate::SysbadgeUc8151<'static>>,
) {
    /*let app = make_static!(picoserve::Router::new()
    .route("/", get(get_counter)));*/
    type WebRouter = impl picoserve::routing::PathRouter<WebState> + 'static;
    let app: &Router<WebRouter, WebState> =
        make_static!(picoserve::Router::new().route("/api/sysinfo", get(get_system_info)));

    let config = make_static!(picoserve::Config {
        start_read_request_timeout: None,
        read_request_timeout: None,
    });

    let buf: &mut [u8] = make_static!([0; 4096]);

    /*unwrap!(embassy_executor::Spawner::for_current_executor().await.spawn(web_task(stack, app, config, WebState {
        badge
    })));*/
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

        loop {
            let (socket_rx, socket_tx) = socket.split();

            match picoserve::serve_with_state(
                app,
                EmbassyTimer,
                config,
                &mut [0; 2048],
                socket_rx,
                socket_tx,
                &WebState { badge },
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
    struct SystemInfo<'a> {
        name: &'a str,
        hid: Option<&'a str>,
        members: u32,
    }
    let state = state.badge.lock().await;
    picoserve::response::Json(SystemInfo {
        name: state.system.name(),
        hid: core::str::from_utf8(state.system.hid())
            .map(|s| s.trim_matches('\0'))
            .map(|s| if s.is_empty() { None } else { Some(s) })
            .ok()
            .flatten(),
        members: state.system.member_count() as u32,
    })
}
