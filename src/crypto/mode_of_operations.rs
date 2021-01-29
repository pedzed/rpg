/// The block cipher mode of operations
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Cfb,
    OpenPgpCfb,
}
