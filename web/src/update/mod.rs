use std::{mem, ptr};

use pkrs::model::PkId;
use sysbadge::system::SystemVec;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Blob, Document, HtmlElement, HtmlInputElement};

const RP2040_FAMILY_ID: u32 = 0xe48bff56;
// HAS TO BE KEP IN SYNC WITH THE VALUE IN `fw/memory.x`
const RP2040_DATA_ADDR: u32 = 0x001F0000;
// HAS TO BE KEP IN SYNC WITH THE VALUE IN `fw/memory.x`
const RP2040_ROM_ADDR: u32 = 0x10000000;

static mut SYSTEM: Option<System> = None;

pub(crate) fn register(document: &Document) -> Result<(), JsValue> {
    if let Some(updater_element) = document.get_element_by_id("sysbadge-updater") {
        updater_element.set_inner_html(include_str!("updater.html"));

        // update button
        {
            let closur = Closure::wrap(Box::new(move || {
                spawn_local(async move {
                    let _system = update().await.unwrap();
                });
            }) as Box<dyn FnMut()>);

            document
                .get_element_by_id("_sysbadge-updater-start")
                .unwrap()
                .add_event_listener_with_callback("click", closur.as_ref().unchecked_ref())
                .unwrap();

            closur.forget();
        }
        // download button
        {
            let closur = Closure::wrap(Box::new(move || {
                spawn_local(async move {
                    if let Some(sys) = unsafe { &SYSTEM } {
                        download_uf2(sys);
                    } else {
                        let system = update().await.unwrap();
                        download_uf2(&system);
                    }
                });
            }) as Box<dyn FnMut()>);

            document
                .get_element_by_id("_sysbadge-updater-download")
                .unwrap()
                .add_event_listener_with_callback("click", closur.as_ref().unchecked_ref())
                .unwrap();

            closur.forget();
        }
        // Input
        {
            let closure = Closure::wrap(Box::new(move || unsafe {
                SYSTEM = None;
            }) as Box<dyn FnMut()>);

            document
                .get_element_by_id("_sysbadge-updater-pkid")
                .unwrap()
                .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                .unwrap();

            closure.forget();
        }
    }

    #[cfg(feature = "badge")]
    {
        spawn_local(async {
            let id = PkId("exmpl".to_string());
            let sys = System::get(id).await.unwrap();

            #[cfg(feature = "badge")]
            sys.set_system();
        })
    }

    Ok(())
}

fn download_uf2(system: &System) {
    let offset = RP2040_ROM_ADDR + RP2040_DATA_ADDR;
    let vec = system.get_system().get_file();

    let download_name = format!("{}.uf2", system.system.name);

    let uint8arr = js_sys::Uint8Array::new(&unsafe { js_sys::Uint8Array::view(&vec) }.into());
    let array = js_sys::Array::new();
    array.push(&uint8arr.buffer());
    let blob = Blob::new_with_u8_array_sequence_and_options(
        &array,
        web_sys::BlobPropertyBag::new().type_("application/octet-stream"),
    )
    .unwrap();
    let download_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    let dlbtn = window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("_sysbadge-updater-download-link")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();
    dlbtn.set_attribute("href", &download_url).unwrap();
    dlbtn.set_attribute("download", &download_name).unwrap();
    dlbtn.click();

    web_sys::Url::revoke_object_url(download_url.as_str()).unwrap();
}

async fn update() -> Result<&'static System, JsValue> {
    let input = window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("_sysbadge-updater-pkid")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();

    let id = PkId(input.value());
    let sys = System::get(id).await?;

    #[cfg(feature = "badge")]
    sys.set_system();

    unsafe { SYSTEM = Some(sys) }

    Ok(unsafe { SYSTEM.as_ref().unwrap_unchecked() })
}

struct System {
    system: SystemVec,
}

impl System {
    async fn get(id: PkId) -> Result<Self, JsValue> {
        let mut updater = sysbadge::system::downloaders::PkDownloader::new();
        updater.client.user_agent = "sysbadge wasm updater".to_string();

        let system = updater.get(&id.0).await?;

        Ok(Self { system })
    }

    fn get_system(&self) -> &SystemVec {
        &self.system
    }

    #[cfg(feature = "badge")]
    fn set_system(&self) {
        unsafe {
            let badge = crate::badge::SYSBADGE.as_mut().unwrap();
            badge.system = Some(self.get_system().clone());
            badge.reset();
            badge.draw().unwrap();
            badge.display.flush().unwrap();
        }
    }
}

fn pad_align_type<T: Sized>(vec: &mut Vec<u8>) {
    let bytes = bytes_to_align(mem::align_of::<T>() as u32, vec.len() as u32);
    vec.extend(core::iter::repeat(0).take(bytes as usize));
}

const fn bytes_to_align_const<const N: u32>(bytes: u32) -> u32 {
    bytes_to_align(N, bytes)
}

const fn bytes_to_align(align: u32, bytes: u32) -> u32 {
    (align - (bytes % align)) % align
}
