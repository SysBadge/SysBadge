use pkrs::client::PkClient;
use pkrs::model::PkId;
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

#[wasm_bindgen]
pub struct Updater {
    id: PkId,
    client: PkClient,
}

#[wasm_bindgen]
impl Updater {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String) -> Self {
        Self {
            id: PkId(id),
            client: PkClient {
                user_agent: "Sysbadge wasm exporter".to_string(),
                ..Default::default()
            },
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.0.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: String) {
        self.id = PkId(id);
    }

    #[wasm_bindgen(js_name = "getSystem")]
    pub async fn get_system(&self) -> Result<JsValue, JsValue> {
        let system = self.client.get_system(&self.id).await?;

        Ok(JsValue::from_serde(&system).unwrap())
    }
}

pub(crate) fn register() -> Result<(), JsValue> {
    let closur = Closure::wrap(Box::new(move || {
        spawn_local(async move {
            let updater = Updater::new("exmpl".to_string());
            let system = updater.get_system().await.unwrap();
            console::log_1(&system);
        });
    }) as Box<dyn FnMut()>);
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("sysbadge-updater-start")
        .unwrap()
        .add_event_listener_with_callback("click", closur.as_ref().unchecked_ref())
        .unwrap();

    closur.forget();

    Ok(())
}
