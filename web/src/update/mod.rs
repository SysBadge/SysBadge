use pkrs::client::PkClient;
use pkrs::model::PkId;
use std::mem::MaybeUninit;
use std::{mem, ptr};
use sysbadge::system::{Member, SystemUf2};
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{console, window, Blob, Document, HtmlButtonElement, HtmlElement, HtmlInputElement};

const RP2040_FAMILY_ID: u32 = 0xe48bff56;
// HAS TO BE KEP IN SYNC WITH THE VALUE IN `fw/memory.x`
const RP2040_DATA_ADDR: u32 = 0x40000;
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
                    let system = update().await.unwrap();
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
    let vec = system.write_vec(RP2040_ROM_ADDR + RP2040_DATA_ADDR);
    let vec = uf2::bin_to_uf2(&vec, RP2040_FAMILY_ID, RP2040_ROM_ADDR + RP2040_DATA_ADDR).unwrap();

    let download_name = if let Some(name) = &system.info.name {
        format!("{}.uf2", name)
    } else {
        format!("{}.uf2", "system")
    };

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
    id: PkId,
    info: pkrs::model::System,
    members: Vec<pkrs::model::Member>,
}

impl System {
    async fn get(id: PkId) -> Result<Self, JsValue> {
        let client = PkClient {
            user_agent: "sysbadge wasm updater".to_string(),
            ..Default::default()
        };

        let info = client.get_system(&id).await?;
        let members = client.get_system_members(&id).await?;

        Ok(Self { id, info, members })
    }

    fn write_vec(&self, offset: u32) -> Vec<u8> {
        let mut ret = Vec::new();

        let mut system = MaybeUninit::zeroed();

        let name_addr = next_after::<u8>(mem::size_of::<SystemUf2>() as u32);
        let name_len = self.info.name.as_ref().map(String::len).unwrap_or(0) as u32;
        let member_addr = next_after::<Member>(name_addr + name_len);
        unsafe {
            ptr::copy_nonoverlapping(
                (offset + name_addr).to_le_bytes().as_ptr(),
                (system.as_mut_ptr() as *mut u8),
                4,
            );
            ptr::copy_nonoverlapping(
                name_len.to_le_bytes().as_ptr(),
                (system.as_mut_ptr() as *mut u8).add(4),
                4,
            );

            ptr::copy_nonoverlapping(
                (offset + member_addr).to_le_bytes().as_ptr(),
                (system.as_mut_ptr() as *mut u8).add(8),
                4,
            );
            ptr::copy_nonoverlapping(
                (self.members.len() as u32).to_le_bytes().as_ptr(),
                (system.as_mut_ptr() as *mut u8).add(12),
                4,
            );

            // TODO: crc16
        }

        let mut system: SystemUf2 = unsafe { system.assume_init() };
        ret.extend(core::iter::repeat(0).take(member_addr as usize));
        unsafe {
            ptr::copy_nonoverlapping(
                &system as *const SystemUf2 as *const u8,
                ret.as_mut_ptr(),
                mem::size_of::<SystemUf2>(),
            );
            ptr::copy_nonoverlapping(
                self.info
                    .name
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                ret.as_mut_ptr().add(name_addr as usize),
                name_len as usize,
            );
        }

        self.write_members(offset, &mut ret);

        ret
    }

    fn write_members(&self, offset: u32, vec: &mut Vec<u8>) {
        let mut start_addr = vec.len();
        let member_bytes = mem::size_of::<Member>() * self.members.len();
        let mut member_end = (start_addr + member_bytes) as u32;
        vec.extend(core::iter::repeat(0).take(member_bytes));

        for member in &self.members {
            member_end += Self::write_member(member_end + offset, start_addr, member, vec);

            start_addr += mem::size_of::<Member>();
        }
    }

    fn write_member(
        offset: u32,
        member_offset: usize,
        member: &pkrs::model::Member,
        vec: &mut Vec<u8>,
    ) -> u32 {
        let name_len = member.name.len() as u32;
        let pronouns_len = member.pronouns.as_ref().map(String::len).unwrap_or(0) as u32;
        let start_addr = vec.len();
        vec.extend(core::iter::repeat(0).take((name_len + pronouns_len) as usize));

        // Write member pointers
        unsafe {
            let member_ptr = vec.as_mut_ptr().add(member_offset);

            ptr::copy_nonoverlapping(offset.to_le_bytes().as_ptr(), member_ptr, 4);
            ptr::copy_nonoverlapping(name_len.to_le_bytes().as_ptr(), member_ptr.add(4), 4);

            ptr::copy_nonoverlapping(
                (offset + name_len).to_le_bytes().as_ptr(),
                member_ptr.add(8),
                4,
            );
            ptr::copy_nonoverlapping(pronouns_len.to_le_bytes().as_ptr(), member_ptr.add(12), 4);
        }

        // write member strings
        unsafe {
            ptr::copy_nonoverlapping(
                member.name.as_ptr(),
                vec.as_mut_ptr().add(start_addr),
                name_len as usize,
            );
            if let Some(pronouns) = &member.pronouns {
                ptr::copy_nonoverlapping(
                    pronouns.as_ptr(),
                    vec.as_mut_ptr().add(start_addr + name_len as usize),
                    pronouns_len as usize,
                );
            }
        }

        name_len + pronouns_len
    }

    #[cfg(feature = "badge")]
    fn get_system(&self) -> SystemUf2 {
        let mut members = Vec::new();
        for member in &self.members {
            members.push(Member::new_str(
                member.name.as_ref(),
                member.pronouns.clone().unwrap_or("".to_string()).as_str(),
            ));
        }
        let members = members.into_boxed_slice();

        SystemUf2::new_from_box(
            self.info
                .name
                .clone()
                .unwrap_or("No system name".to_string())
                .into_boxed_str(),
            members,
        )
    }

    #[cfg(feature = "badge")]
    fn set_system(&self) {
        let sys = self.get_system();

        unsafe {
            crate::badge::SYSTEM = sys;
            let badge = crate::badge::SYSBADGE.as_mut().unwrap();
            badge.system = &crate::badge::SYSTEM;
            badge.reset();
            badge.draw().unwrap();
            badge.display.flush().unwrap();
        }
    }
}

const fn next_after<T: Sized>(curr: u32) -> u32 {
    let pad = bytes_to_align(mem::align_of::<T>() as u32, curr);
    curr + pad
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
