//! Cross-platform HID abstraction for Razer BlackShark headset control.
//!
//! Uses `hidapi` on all platforms. On macOS this wraps IOHIDManager under the
//! hood; opening by vendor ID (0x1532) ensures only Razer devices are matched,
//! so Input Monitoring permission is not required.

use hidapi::{HidApi, HidDevice};
use strop_proto::{decode, Message, PAYLOAD_LEN, REPORT_ID};
use std::sync::OnceLock;
use std::time::Duration;

/// Global cached HidApi instance. Created once before the winit event loop
/// to avoid macOS's hid_enumerate re-entering the CoreFoundation run loop
/// while winit is handling an event.
fn hid_api() -> Result<&'static HidApi, HidError> {
    static API: OnceLock<HidApi> = OnceLock::new();
    Ok(API.get_or_init(|| {
        HidApi::new().expect("hidapi init failed — is your Razer dongle plugged in?")
    }))
}

// Razer's USB vendor ID.
pub const RAZER_VID: u16 = 0x1532;

// BlackShark V3 family product IDs.
pub const V3_PRO_DONGLE: u16 = 0x0577;
pub const V3_PRO_WIRED: u16 = 0x0576;
pub const V3_DONGLE: u16 = 0x057A;
pub const V3_WIRED: u16 = 0x0579;

/// All known BlackShark V3 product IDs.
pub const ALL_PIDS: &[u16] = &[V3_PRO_DONGLE, V3_PRO_WIRED, V3_DONGLE, V3_WIRED];

/// V3 Pro-only product IDs.
pub const V3_PRO_PIDS: &[u16] = &[V3_PRO_DONGLE, V3_PRO_WIRED];

/// V3 (non-Pro) product IDs.
pub const V3_PIDS: &[u16] = &[V3_DONGLE, V3_WIRED];

/// Whether a product ID belongs to the V3 (non-Pro).
pub fn is_v3(pid: u16) -> bool {
    pid == V3_DONGLE || pid == V3_WIRED
}

/// Transport name for a product ID.
pub fn transport(pid: u16) -> &'static str {
    match pid {
        V3_PRO_DONGLE | V3_DONGLE => "dongle",
        V3_PRO_WIRED | V3_WIRED => "wired",
        _ => "unknown",
    }
}

/// Information about one HID interface.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub product: String,
    pub manufacturer: String,
    pub serial: String,
    pub usage_page: Option<u16>,
    pub usage: Option<u16>,
}

/// Error type for HID operations.
#[derive(Debug)]
pub enum HidError {
    NotFound(String),
    OpenFailed(String),
    WriteFailed(String),
    ReadFailed(String),
    Timeout,
    Unsupported(String),
    Platform(String),
}

impl std::fmt::Display for HidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "device not found: {s}"),
            Self::OpenFailed(s) => write!(f, "open failed: {s}"),
            Self::WriteFailed(s) => write!(f, "write failed: {s}"),
            Self::ReadFailed(s) => write!(f, "read failed: {s}"),
            Self::Timeout => write!(f, "no reply before timeout"),
            Self::Unsupported(s) => write!(f, "unsupported: {s}"),
            Self::Platform(s) => write!(f, "platform error: {s}"),
        }
    }
}

impl std::error::Error for HidError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_correct() {
        assert_eq!(RAZER_VID, 0x1532);
        assert_eq!(V3_PRO_DONGLE, 0x0577);
        assert_eq!(V3_PRO_WIRED, 0x0576);
        assert_eq!(V3_DONGLE, 0x057A);
        assert_eq!(V3_WIRED, 0x0579);
    }

    #[test]
    fn is_v3_positive() {
        assert!(is_v3(V3_DONGLE));
        assert!(is_v3(V3_WIRED));
    }

    #[test]
    fn is_v3_negative() {
        assert!(!is_v3(V3_PRO_DONGLE));
        assert!(!is_v3(V3_PRO_WIRED));
        assert!(!is_v3(0x0000));
    }

    #[test]
    fn all_pids_includes_all() {
        assert!(ALL_PIDS.contains(&V3_PRO_DONGLE));
        assert!(ALL_PIDS.contains(&V3_PRO_WIRED));
        assert!(ALL_PIDS.contains(&V3_DONGLE));
        assert!(ALL_PIDS.contains(&V3_WIRED));
        assert_eq!(ALL_PIDS.len(), 4);
    }

    #[test]
    fn v3_pro_pids_correct() {
        assert_eq!(V3_PRO_PIDS, &[V3_PRO_DONGLE, V3_PRO_WIRED]);
    }

    #[test]
    fn v3_pids_correct() {
        assert_eq!(V3_PIDS, &[V3_DONGLE, V3_WIRED]);
    }

    #[test]
    fn transport_names() {
        assert_eq!(transport(V3_PRO_DONGLE), "dongle");
        assert_eq!(transport(V3_DONGLE), "dongle");
        assert_eq!(transport(V3_PRO_WIRED), "wired");
        assert_eq!(transport(V3_WIRED), "wired");
        assert_eq!(transport(0x0000), "unknown");
    }

    #[test]
    fn display_not_found() {
        let e = HidError::NotFound("test".into());
        assert_eq!(e.to_string(), "device not found: test");
    }

    #[test]
    fn display_open_failed() {
        let e = HidError::OpenFailed("test".into());
        assert_eq!(e.to_string(), "open failed: test");
    }

    #[test]
    fn display_write_failed() {
        let e = HidError::WriteFailed("test".into());
        assert_eq!(e.to_string(), "write failed: test");
    }

    #[test]
    fn display_read_failed() {
        let e = HidError::ReadFailed("test".into());
        assert_eq!(e.to_string(), "read failed: test");
    }

    #[test]
    fn display_timeout() {
        let e = HidError::Timeout;
        assert_eq!(e.to_string(), "no reply before timeout");
    }

    #[test]
    fn display_unsupported() {
        let e = HidError::Unsupported("test".into());
        assert_eq!(e.to_string(), "unsupported: test");
    }

    #[test]
    fn display_platform() {
        let e = HidError::Platform("test".into());
        assert_eq!(e.to_string(), "platform error: test");
    }

    #[test]
    fn device_info_defaults() {
        let info = DeviceInfo {
            vendor_id: 0x1532,
            product_id: 0x0577,
            product: "BlackShark V3 Pro".into(),
            manufacturer: "Razer".into(),
            serial: "".into(),
            usage_page: Some(0xFF14),
            usage: Some(0x01),
        };
        assert_eq!(info.vendor_id, 0x1532);
        assert_eq!(info.product_id, 0x0577);
    }
}
pub struct Device {
    inner: HidDevice,
    pub info: DeviceInfo,
}

impl Device {
    /// Send an output report with the given report ID.
    ///
    /// `data` is the payload without the report ID. The report ID is prefixed
    /// automatically (hidapi requires it in the first byte for numbered reports).
    pub fn set_report(&self, report_id: u8, data: &[u8]) -> Result<(), HidError> {
        let mut framed = vec![report_id];
        framed.extend_from_slice(data);
        self.inner
            .write(&framed)
            .map_err(|e| HidError::WriteFailed(format!("write: {e}")))?;
        Ok(())
    }

    /// Send a command and wait for the device's reply.
    ///
    /// This is the core protocol operation: encodes the message, writes it as
    /// an output report, then reads the response from the interrupt IN endpoint.
    pub fn request(&self, msg: &Message, timeout: Duration) -> Result<Message, HidError> {
        let payload = msg
            .encode()
            .map_err(|e| HidError::WriteFailed(format!("encode: {e}")))?;

        // Write: HID report ID (0x02) + 63-byte payload.
        let mut framed = vec![REPORT_ID];
        framed.extend_from_slice(&payload);
        self.inner
            .write(&framed)
            .map_err(|e| HidError::WriteFailed(format!("write: {e}")))?;

        // Read: maximum 64 bytes (report ID + 63 payload).
        let mut buf = vec![0u8; PAYLOAD_LEN + 1];
        let n = self
            .inner
            .read_timeout(&mut buf, timeout.as_millis() as i32)
            .map_err(|e| HidError::ReadFailed(format!("read: {e}")))?;

        if n == 0 {
            return Err(HidError::Timeout);
        }

        buf.truncate(n);
        decode(&buf).map_err(|e| HidError::ReadFailed(format!("decode: {e}")))
    }

    /// Listen for input reports without sending anything.
    pub fn listen(&self, timeout: Duration) -> Result<Vec<Vec<u8>>, HidError> {
        let deadline = std::time::Instant::now() + timeout;
        let mut reports = Vec::new();

        loop {
            let remaining = deadline
                .checked_duration_since(std::time::Instant::now())
                .unwrap_or(Duration::ZERO);
            if remaining.is_zero() {
                break;
            }
            let mut buf = vec![0u8; PAYLOAD_LEN + 1];
            let n = self
                .inner
                .read_timeout(&mut buf, remaining.as_millis() as i32)
                .map_err(|e| HidError::ReadFailed(format!("read: {e}")))?;
            if n > 0 {
                buf.truncate(n);
                reports.push(buf);
            }
        }
        Ok(reports)
    }
}

/// Enumerate all Razer HID devices (VID = 0x1532).
pub fn list() -> Result<Vec<DeviceInfo>, HidError> {
    let api = hid_api()?;
    let mut out = Vec::new();
    for d in api.device_list() {
        if d.vendor_id() == RAZER_VID {
            out.push(DeviceInfo {
                vendor_id: d.vendor_id(),
                product_id: d.product_id(),
                product: d.product_string().unwrap_or("").to_string(),
                manufacturer: d.manufacturer_string().unwrap_or("").to_string(),
                serial: d.serial_number().unwrap_or("").to_string(),
                usage_page: Some(d.usage_page()),
                usage: Some(d.usage()),
            });
        }
    }
    Ok(out)
}

/// Open the first device matching the given product IDs under Razer VID.
/// Convenience wrapper around `open(RAZER_VID, pids)`.
pub fn open_razer(pids: &[u16]) -> Result<Device, HidError> {
    open(RAZER_VID, pids)
}

/// Force the HID API to initialize. Call before starting a winit/eframe event
/// loop to avoid macOS's hid_enumerate re-entrancy crash.
pub fn init() -> Result<(), HidError> {
    hid_api().map(|_| ())
}

/// Check whether any Razer headset is present on the bus.
///
/// Uses a fast enumeration poll. Suitable for watch loops.
pub fn present() -> bool {
    match list() {
        Ok(devices) => devices.iter().any(|d| ALL_PIDS.contains(&d.product_id)),
        Err(_) => false,
    }
}
pub fn open(vid: u16, pids: &[u16]) -> Result<Device, HidError> {
    let api = hid_api()?;

    for pid in pids {
        if let Ok(inner) = api.open(vid, *pid) {
            let info = DeviceInfo {
                vendor_id: vid,
                product_id: *pid,
                product: inner
                    .get_product_string()
                    .unwrap_or(None)
                    .unwrap_or_default(),
                manufacturer: inner
                    .get_manufacturer_string()
                    .unwrap_or(None)
                    .unwrap_or_default(),
                serial: inner
                    .get_serial_number_string()
                    .unwrap_or(None)
                    .unwrap_or_default(),
                usage_page: None,
                usage: None,
            };
            return Ok(Device { inner, info });
        }
    }

    Err(HidError::NotFound(format!(
        "no Razer device {vid:04x}:{:?} on the bus",
        pids.iter().map(|p| format!("{p:04x}")).collect::<Vec<_>>()
    )))
}
