use core::cell::{Cell, RefCell};

use defmt::*;
use nrf_softdevice::{
    ble::{
        gatt_server::{self, RegisterError},
        security::SecurityHandler,
        Connection, EncryptionInfo, IdentityKey, MasterId, SecurityMode, Uuid,
    },
    Softdevice,
};

const BATTERY_SERVICE: Uuid = Uuid::new_16(0x180f);
const BATTERY_LEVEL: Uuid = Uuid::new_16(0x2a19);

#[derive(Debug, Clone, Copy)]
struct Peer {
    master_id: MasterId,
    key: EncryptionInfo,
    peer_id: IdentityKey,
}

pub struct Bonder {
    peer: Cell<Option<Peer>>,
    sys_attrs: RefCell<heapless::Vec<u8, 62>>,
}

impl Default for Bonder {
    fn default() -> Self {
        Self {
            peer: Cell::new(None),
            sys_attrs: RefCell::new(heapless::Vec::new()),
        }
    }
}

impl SecurityHandler for Bonder {
    fn io_capabilities(&self) -> nrf_softdevice::ble::security::IoCapabilities {
        nrf_softdevice::ble::security::IoCapabilities::DisplayOnly // TODO: DisplayYesNo when adding display
    }

    fn can_bond(&self, _conn: &nrf_softdevice::ble::Connection) -> bool {
        true
    }

    fn display_passkey(&self, passkey: &[u8; 6]) {
        info!("The passkey is {:a}", passkey);
    }

    fn on_bonded(
        &self,
        _conn: &nrf_softdevice::ble::Connection,
        master_id: MasterId,
        key: EncryptionInfo,
        peer_id: IdentityKey,
    ) {
        debug!("Storing bond for: id: {}, key: {}", master_id, key);

        self.sys_attrs.borrow_mut().clear();
        self.peer.set(Some(Peer {
            master_id,
            key,
            peer_id,
        }));
    }

    fn get_key(
        &self,
        _conn: &nrf_softdevice::ble::Connection,
        master_id: MasterId,
    ) -> Option<EncryptionInfo> {
        debug!("getting bond for: id: {}", master_id);

        self.peer
            .get()
            .and_then(|peer| (master_id == peer.master_id).then_some(peer.key))
    }

    fn save_sys_attrs(&self, conn: &nrf_softdevice::ble::Connection) {
        debug!("saving system attributes for: {}", conn.peer_address());

        if let Some(peer) = self.peer.get() {
            if peer.peer_id.is_match(conn.peer_address()) {
                let mut sys_attrs = self.sys_attrs.borrow_mut();
                let capacity = sys_attrs.capacity();
                unwrap!(sys_attrs.resize(capacity, 0));
                let len = unwrap!(gatt_server::get_sys_attrs(conn, &mut sys_attrs)) as u16;
                sys_attrs.truncate(usize::from(len));
                // FIXME:
                // In a real application you would want to signal another task to permanently store sys_attrs for this connection's peer
            }
        }
    }

    fn load_sys_attrs(&self, conn: &nrf_softdevice::ble::Connection) {
        let addr = conn.peer_address();
        debug!("loading system attributes for: {}", addr);

        let attrs = self.sys_attrs.borrow();
        let attrs = if self
            .peer
            .get()
            .map(|peer| peer.peer_id.is_match(addr))
            .unwrap_or(false)
        {
            (!attrs.is_empty()).then_some(attrs.as_slice())
        } else {
            None
        };

        unwrap!(gatt_server::set_sys_attrs(conn, attrs));
    }
}

pub struct BatteryService {
    value_handle: u16,
    cccd_handle: u16,
}

impl BatteryService {
    pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
        let mut service_builder = gatt_server::builder::ServiceBuilder::new(sd, BATTERY_SERVICE)?;

        let attr =
            gatt_server::characteristic::Attribute::new(&[0u8]).security(SecurityMode::JustWorks);
        let metdata = gatt_server::characteristic::Metadata::new(
            gatt_server::characteristic::Properties::new()
                .read()
                .notify(),
        );
        let characteristic_builder =
            service_builder.add_characteristic(BATTERY_LEVEL, attr, metdata)?;
        let charasteristic_handle = characteristic_builder.build();

        let _service_handle = service_builder.build();

        Ok(Self {
            value_handle: charasteristic_handle.value_handle,
            cccd_handle: charasteristic_handle.cccd_handle,
        })
    }

    pub fn battery_level_get(&self, sd: &Softdevice) -> Result<u8, gatt_server::GetValueError> {
        let buf = &mut [0u8];
        gatt_server::get_value(sd, self.value_handle, buf)?;
        Ok(buf[0])
    }

    pub fn battery_level_set(
        &self,
        sd: &Softdevice,
        val: u8,
    ) -> Result<(), gatt_server::SetValueError> {
        gatt_server::set_value(sd, self.value_handle, &[val])
    }
    pub fn battery_level_notify(
        &self,
        conn: &Connection,
        val: u8,
    ) -> Result<(), gatt_server::NotifyValueError> {
        gatt_server::notify_value(conn, self.value_handle, &[val])
    }

    pub fn on_write(&self, handle: u16, data: &[u8]) {
        if handle == self.cccd_handle && !data.is_empty() {
            info!("battery notifications: {}", (data[0] & 0x01) != 0);
        }
    }
}

pub struct Server {
    bas: BatteryService,
}

impl Server {
    pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
        let bas = BatteryService::new(sd)?;

        Ok(Self { bas })
    }
}

impl gatt_server::Server for Server {
    type Event = ();

    fn on_write(
        &self,
        conn: &nrf_softdevice::ble::Connection,
        handle: u16,
        op: gatt_server::WriteOp,
        offset: usize,
        data: &[u8],
    ) -> Option<Self::Event> {
        if handle == self.bas.cccd_handle {
            self.bas.on_write(handle, data);
        }

        None
    }
}

#[embassy_executor::task]
pub async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}
