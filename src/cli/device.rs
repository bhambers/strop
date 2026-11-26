//! Shared device helpers for CLI commands (read slot, write slot, send).

use strop_hid::Device;
use strop_proto::{self as proto, Eq};
use std::time::Duration;

/// Gap Synapse leaves between transfers.
pub const INTER_FRAME: Duration = Duration::from_millis(30);

/// Reply timeout.
pub const TIMEOUT: Duration = Duration::from_secs(3);

/// Number of retry attempts for flaky NAKs.
pub const MAX_ATTEMPTS: u32 = 10;
pub const RETRY_BACKOFF: Duration = Duration::from_millis(40);

/// Send a message and drain the reply.
pub fn send(device: &Device, msg: &proto::Message) -> Result<(), String> {
    let reply = retry(|| device.request(msg, TIMEOUT))
        .map_err(|e| format!("send: {e}"))?;
    if reply.status != proto::STATUS_SUCCESS {
        return Err(format!("device replied status {:#04X}", reply.status));
    }
    std::thread::sleep(INTER_FRAME);
    Ok(())
}

/// Write the three-step EQ sequence: activate, set bands, commit.
pub fn write_slot(device: &Device, slot: u8, eq: Eq) -> Result<(), String> {
    let steps: Vec<(&str, proto::Message)> = vec![
        ("activateSlot", proto::activate_slot(slot)),
        ("setCustomerEQBand", proto::set_bands(slot, eq)),
        ("commitFootstepScaling", proto::commit_bands(slot, eq)),
    ];
    for (label, msg) in &steps {
        let reply = retry(|| device.request(msg, TIMEOUT))
            .map_err(|e| format!("{label}: {e}"))?;
        if reply.status != proto::STATUS_SUCCESS {
            return Err(format!("{label}: device replied status {:#04X}", reply.status));
        }
        std::thread::sleep(INTER_FRAME);
    }
    Ok(())
}

/// Select a slot as the active one.
pub fn select_slot(device: &Device, slot: u8) -> Result<(), String> {
    send(device, &proto::select_preset(slot))
}

/// Read one EQ slot from the device (order info + bands).
pub fn read_slot(device: &Device, position: u8) -> Result<(proto::Order, Eq), String> {
    let order_reply = retry(|| device.request(&proto::order_info(position), TIMEOUT))
        .map_err(|e| format!("order_info slot {position}: {e}"))?;
    let order = proto::parse_order(&order_reply.args)
        .map_err(|e| format!("parse order slot {position}: {e}"))?;

    if proto::unavailable(&order_reply.args) {
        return Err("headset is off or out of range".into());
    }

    let bands_reply = retry(|| device.request(&proto::get_bands(position), TIMEOUT))
        .map_err(|e| format!("get_bands slot {position}: {e}"))?;
    let eq = Eq::from_bytes(&bands_reply.args)
        .map_err(|e| format!("parse bands slot {position}: {e}"))?;

    Ok((order, eq))
}

/// Retry a fallible operation.
fn retry<F, T>(op: F) -> Result<T, String>
where
    F: Fn() -> Result<T, strop_hid::HidError>,
{
    let mut last = String::new();
    for _attempt in 1..=MAX_ATTEMPTS {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) => {
                last = format!("{e}");
                std::thread::sleep(RETRY_BACKOFF);
            }
        }
    }
    Err(format!("failed after {MAX_ATTEMPTS} attempts, last: {last}"))
}
