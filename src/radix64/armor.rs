// Example:
//
// -----BEGIN PGP MESSAGE-----
// Version: OpenPrivacy 0.99
//
// yDgBO22WxBHv7O8X7O/jygAEzol56iUKiXmV+XmpCtmpqQUKiQrFqclFqUDBovzS
// vBSFjNSiVHsuAA==
// =njUN
// -----END PGP MESSAGE-----
mod armor_checksums;
mod armor_data_types;
mod armor_data_headers;

pub mod armor_reader;
pub mod armor_writer;

use std::collections::HashMap;

use super::armor::armor_data_headers::ArmorDataHeader;
use super::coding::Radix64;

type ArmorDataHeaderMap = HashMap<ArmorDataHeader, Vec<String>>;
type ArmorData = Radix64;

pub const LINE_ENDING: &str = "\r\n";
