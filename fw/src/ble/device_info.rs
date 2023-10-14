use defmt::*;
use nrf_softdevice::ble::gatt_server::builder::ServiceBuilder;
use nrf_softdevice::ble::gatt_server::characteristic::{Attribute, Metadata, Properties};
use nrf_softdevice::ble::gatt_server::{CharacteristicHandles, RegisterError};
use nrf_softdevice::ble::Uuid;
use nrf_softdevice::Softdevice;

const DEVICE_INFORMATION_SERVICE: Uuid = Uuid::new_16(0x180A);

const MODEL_NUMBER_STRING: Uuid = Uuid::new_16(0x2A24);
const SERIAL_NUMBER_STRING: Uuid = Uuid::new_16(0x2A25);
const FIRMWARE_REVISION_STRING: Uuid = Uuid::new_16(0x2A26);
const HARDWARE_REVISION_STRING: Uuid = Uuid::new_16(0x2A27);
const MANUFACTURER_NAME_STRING: Uuid = Uuid::new_16(0x2A29);

#[derive(Debug, Default, defmt::Format)]
pub struct DeviceInformation {
    pub model_number: &'static str,
    pub serial_number: &'static str,
    pub fw_rev: &'static str,
    pub hw_rev: &'static str,
    pub manufacturer_name: &'static str,
}

pub struct DeviceInformationService {}

impl DeviceInformationService {
    pub fn new(sd: &mut Softdevice, info: DeviceInformation) -> Result<Self, RegisterError> {
        let mut sb = ServiceBuilder::new(sd, DEVICE_INFORMATION_SERVICE)?;

        // TODO: PnP_ID?
        Self::add_str_characteristic(&mut sb, MODEL_NUMBER_STRING, info.model_number)?;

        Self::add_str_characteristic(&mut sb, SERIAL_NUMBER_STRING, info.serial_number)?;
        Self::add_str_characteristic(&mut sb, FIRMWARE_REVISION_STRING, info.fw_rev)?;
        Self::add_str_characteristic(&mut sb, HARDWARE_REVISION_STRING, info.hw_rev)?;
        Self::add_str_characteristic(&mut sb, MANUFACTURER_NAME_STRING, info.manufacturer_name)?;

        let _service_handle = sb.build();

        Ok(DeviceInformationService {})
    }

    fn add_str_characteristic(
        sb: &mut ServiceBuilder,
        uuid: Uuid,
        val: &'static str,
    ) -> Result<CharacteristicHandles, RegisterError> {
        let attr = Attribute::new(val);
        let md = Metadata::new(Properties::new().read());
        Ok(sb.add_characteristic(uuid, attr, md)?.build())
    }
}
