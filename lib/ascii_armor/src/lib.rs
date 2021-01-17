/// Crc24 calculation
pub mod crc24;

mod checksum;
mod data_headers;
mod data_types;
mod errors;
mod reader;
mod writer;

pub use checksum::ArmorChecksum;
pub use data_headers::ArmorDataHeader;
pub use data_types::ArmorDataType;
pub use errors::ArmorError;
pub use reader::ArmorReader;
pub use writer::ArmorWriter;
pub use writer::ArmorWriterBuilder;

use std::collections::HashMap;
pub(crate) type ArmorDataHeaderMap = HashMap<ArmorDataHeader, Vec<String>>;

pub(crate) const LINE_ENDING: &str = "\r\n";
