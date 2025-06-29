use rupnp::{Device, ssdp::URN};
use std::{error::Error, time::Duration};
use futures_util::StreamExt;
use crate::dlna::models::DlnaDevice;

const EXTRA: &[&str; 1] = &["roomName"];

pub  async fn discover_devices() -> Result<Vec<DlnaDevice>, Box<dyn Error>> {
    let mut dlna_devices = Vec::new();
    
    let search_target = URN::device("schema-upnp-org", "MediaServer", 1).into();
    let devices = rupnp::discover_with_properties(&search_target, Duration::from_secs(3), None, EXTRA).await?;
    
    futures_util::pin_mut!(devices);
    while let Some(device) = devices.next().await {
        match device {
            Ok(device) => {
                let dlna_device = convert_device(&device).await?;
                dlna_devices.push(dlna_device);
            }
            Err(e) => log::error!("Error discovering devices: {}", e),
        }
    }
    
    Ok(dlna_devices)
}

async fn convert_device(device: &Device) -> Result<DlnaDevice, Box<dyn Error>> {
    let location = device.get_extra_property(EXTRA[0]).unwrap_or_default();
    
    Ok(DlnaDevice {
        name: device.friendly_name().to_string(),
        uri: device.url().to_string(),
        udn: device.udn().to_string(),
        device_type: device.device_type().to_string(),
        location: location.to_string(),
    })
}