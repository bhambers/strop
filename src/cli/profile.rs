//! `strop profile` — read every EQ slot back off the headset.

use clap::Args;
use strop_hid::{self as hid, RAZER_VID, ALL_PIDS};
use strop_proto::{self as proto, Eq, SLOTS};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Args)]
pub struct ProfileArgs;

impl ProfileArgs {
    pub fn run(&self) -> Result<(), String> {
        let device = hid::open(RAZER_VID, ALL_PIDS)
            .map_err(|e| format!("open: {e}"))?;

        let is_v3 = hid::is_v3(device.info.product_id);

        // Read every slot to find the active one and collect all data.
        let mut slots: Vec<(proto::Order, Eq)> = Vec::with_capacity(SLOTS);
        let mut active_index: Option<u8> = None;

        for pos in 0..SLOTS as u8 {
            // OrderInfo selects the slot and returns its metadata.
            let order_reply = device.request(&proto::order_info(pos), TIMEOUT)
                .map_err(|e| format!("order_info slot {pos}: {e}"))?;
            let order = proto::parse_order(&order_reply.args)
                .map_err(|e| format!("parse order slot {pos}: {e}"))?;

            if order.active {
                active_index = Some(pos);
            }

            // Read the EQ bands at this cursor position.
            // The reply args are [slot_idx, b0, b1, ..., b9] — skip the slot byte.
            let bands_reply = device.request(&proto::get_bands(pos), TIMEOUT)
                .map_err(|e| format!("get_bands slot {pos}: {e}"))?;
            if bands_reply.args.len() < 1 + proto::BANDS {
                return Err(format!(
                    "slot {pos}: expected {} band bytes, got {}",
                    proto::BANDS,
                    bands_reply.args.len().saturating_sub(1),
                ));
            }
            let eq = Eq::from_bytes(&bands_reply.args[1..=proto::BANDS])
                .map_err(|e| format!("parse bands slot {pos}: {e}"))?;

            slots.push((order, eq));
        }

        if let Some(idx) = active_index {
            println!("active = {idx}");
        }
        if !is_v3 {
            println!();
        }

        for (pos, (order, eq)) in slots.iter().enumerate() {
            print_slot(pos as u8, order, eq, is_v3);
        }

        Ok(())
    }
}

fn print_slot(pos: u8, order: &proto::Order, eq: &Eq, _is_v3: bool) {
    let marker = if order.active { "*" } else { " " };
    let custom = if order.custom { " custom" } else { "" };

    // Look up the preset name from the library.
    let name = proto::library_name(order.cloud_id)
        .unwrap_or("");

    // Format bands as comma-separated with sign.
    let bands: String = eq.0.iter()
        .map(|v| format!("{:+.0}", v))
        .collect::<Vec<_>>()
        .join(",");

    if !name.is_empty() {
        println!(" {marker}{pos}  {bands} {} cloud={}{custom}", name, order.cloud_id);
    } else if order.cloud_id != 0 {
        println!(" {marker}{pos}  {bands} cloud={}{custom}", order.cloud_id);
    } else {
        println!(" {marker}{pos}  {bands}{custom}");
    }
}
