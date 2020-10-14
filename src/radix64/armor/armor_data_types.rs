#[derive(Debug, PartialEq)]
pub enum ArmorDataType {
    /// Used for signed, encrypted, or compressed files.
    PgpMessage,

    /// Used for armoring public keys.
    PgpPublicKeyBlock,

    /// Used for armoring private keys.
    PgpPrivateKeyBlock,

    /// Used for multi-part messages, where the armor is split amongst Y
    /// parts, and this is the Xth part out of Y.
    PgpMessagePartXy(u8, u8),

    /// Used for multi-part messages, where this is the Xth part of an
    /// unspecified number of parts.  Requires the MESSAGE-ID Armor
    /// Header to be used.
    PgpMessagePartX(u8),

    /// Used for detached signatures, OpenPGP/MIME signatures, and
    /// cleartext signatures.  Note that PGP 2.x uses BEGIN PGP MESSAGE
    /// for detached signatures.
    PgpSignature,

    // https://tools.ietf.org/html/rfc4880#section-7
    PgpSignedMessage,

    // NOTE: GnuPG specific
    // https://github.com/gpg/gnupg/blob/master/g10/armor.c#L84-L93
    PgpArmoredFile,

    /// When exporting a private key, PGP 2.x generates the header "BEGIN
    /// PGP SECRET KEY BLOCK" instead of "BEGIN PGP PRIVATE KEY BLOCK".
    /// All previous versions ignore the implied data type, and look
    /// directly at the packet data type.
    PgpSecretKeyBlock,
}
