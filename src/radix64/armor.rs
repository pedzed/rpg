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
mod armor_headers;
pub mod armor_writer;

// impl Armor {
//
//     pub fn parse_str(armor: &str) -> Self {
//         let armor = normalize_armor(armor);

//         let armor_data_type: &str = extract_armor_data_type(&armor);
//         let armor_data_type: ArmorDataType = detect_armor_data_type(armor_data_type)
//             .expect("Armor data type not found.")
//         ;

//         let armor = Self::new(armor_data_type);

//         armor
//     }
// }

// fn normalize_armor(armor: &str) -> String {
//     armor.trim()
//         .replace("\r\n", "⏎")
//         .replace("\r", "⏎")
//         .replace("\n", "⏎")
//         .replace("⏎", LINE_ENDING)
// }

// fn extract_armor_data_type(armor: &str) -> &str {
//     armor.lines().nth(0).unwrap()
// }

// fn detect_armor_data_type(armor_header_line: &str) -> Option<ArmorDataType> {
//     let stripped_header_line = armor_header_line
//         .replace("-", "")
//         .replace("BEGIN", "")
//         .trim()
//         .to_string()
//     ;

//     if stripped_header_line == "PGP MESSAGE" {
//         Some(ArmorDataType::PgpMessage)
//     } else if stripped_header_line == "PGP PUBLIC KEY BLOCK" {
//         Some(ArmorDataType::PgpPublicKeyBlock)
//     } else if stripped_header_line == "PGP PRIVATE KEY BLOCK" {
//         Some(ArmorDataType::PgpPrivateKeyBlock)
//     } else if stripped_header_line.starts_with("PGP MESSAGE, PART ") {
//         let parts: Vec<&str> = stripped_header_line["PGP MESSAGE, PART ".len()..]
//             .split("/")
//             .collect()
//         ;

//         if parts.len() == 1 {
//             let x: u8 = parts[0].parse().unwrap();
//             Some(ArmorDataType::PgpMessagePartX(x))
//         } else if parts.len() == 2 {
//             let x: u8 = parts[0].parse().unwrap();
//             let y: u8 = parts[1].parse().unwrap();
//             Some(ArmorDataType::PgpMessagePartXY(x, y))
//         } else {
//             None
//         }
//     } else if stripped_header_line == "PGP SIGNATURE" {
//         // NOTE: Exceptional case
//         // TODO
//         // Some(ArmorDataType::PgpSignature)
//         None
//     } else if stripped_header_line == "PGP SIGNED MESSAGE" {
//         Some(ArmorDataType::PgpSignedMessage)
//     } else if stripped_header_line == "PGP ARMORED FILE" {
//         Some(ArmorDataType::PgpArmoredFile)
//     } else if stripped_header_line == "PGP SECRET KEY BLOCK" {
//         Some(ArmorDataType::PgpSecretKeyBlock)
//     } else {
//         None
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn parse_pgp_message_header_line_from_armor_str() {
//         let armor_str = "-----BEGIN PGP MESSAGE-----";

//         let armor = Armor::parse_str(armor_str);
//         assert_eq!(armor.data_type, ArmorDataType::PgpMessage);
//     }

//     #[test]
//     fn parse_pgp_message_part_x_header_line_from_armor_str() {
//         let armor_str = "-----BEGIN PGP MESSAGE, PART 2-----\r\n";

//         let armor = Armor::parse_str(armor_str);
//         assert_eq!(armor.data_type, ArmorDataType::PgpMessagePartX(2));
//     }

//     #[test]
//     fn parse_pgp_message_part_xy_header_line_from_armor_str() {
//         let armor_str = "-----BEGIN PGP MESSAGE, PART 2/3-----\r\n";

//         let armor = Armor::parse_str(armor_str);
//         assert_eq!(armor.data_type, ArmorDataType::PgpMessagePartXY(2, 3));
//     }

//     #[test]
//     fn parse_armor_str() {
//         // Note that line endings are purposefully different
//         let armor_str = "
//             -----BEGIN PGP MESSAGE-----\r\
//             Version: OpenPrivacy 0.99\n\
//             Comment: Note that some transport methods are sensitive to line length.  While\r\n\
//             Comment: there is a limit of 76 characters for the Radix-64 data (Section\r\n\
//             Comment: 6.3), there is no limit to the length of Armor Headers.  Care should\r\n\
//             Comment: be taken that the Armor Headers are short enough to survive\r\n\
//             Comment: transport.  One way to do this is to repeat an Armor Header key\r\n\
//             Comment: multiple times with different values for each so that no one line is\r\n\
//             Comment: overly long.\r\n\
//             \r\n\
//             iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAMAAAD04JH5AAAAM1BMVEUAk90NmN48\r\n\
//             reaV0/F2xe0Ane284/f///8AjdsAmOb2+/7W7vro9fwno+Ku3fVStehcuumhrI4O\r\n\
//             AAAFcklEQVR42u2b2ZLcIAxFAWEas///14bF7gav4G2qUtZDMnnI3IMQSAaBPidM\r\n\
//             KClZMKmEOPg70MH/pxg4ojFCqPeGENaWghQPAUgguu+7LigPFv+JNDXidgAFBAex\r\n\
//             Jeu7XlN5K4CkXh1tWN8hx24DUF5+HDsP9vuJ/xg6ZOUtAAL0IO/1MHEUTFwDBoD6\r\n\
//             gMwgOgziegBGkrzXIdSomXcMJWhk6DsiLwYQ3vtJPgsz6UdPqXM0esNvCj5Av04w\r\n\
//             lwKoNHwvD2pci87vAzwzjImlFn8XJlwIwHQa/ji3ficI2oiXS6CIxb6jlwEYnIbv\r\n\
//             0uiZw5yjfashqAKAuOX54afRW1SjXkmA6vW1TMFYLV9FUAFgkj5RKRga5EMgwGkA\r\n\
//             hjN9QG36qMfsJIDSyf9BX7hG+UCg1SkAQboYfzLuBe36PgzcKQCa1r9Z0+9TPXI8\r\n\
//             DNBeAEZ9GueCz5J/j7U3HH7ccAFWhwFUCkAtwlyU+l2PHTDli0FfGfo8hNcZNtfi\r\n\
//             NoD7ToCwvJS3k3woqV5HkAcBWJoAG2KBFxNP2FKthldqpY4cA0grIPIDLyZ1JayU\r\n\
//             WwnInh0CgG50gMz3n61aA5YJNlywASB0+mWsDMDOit19s8EFaNcBpAyAbf01gs62\r\n\
//             AwwO4PBhhTNFVe6aGpbNAGkPQljlE7C3s/88V7sXrAPYMQTzFdDXVJqkW8pJohEg\r\n\
//             bYJ+E1K4ZiqLPQktTMIaOtpxJFZZBPa4rtinXX1SRDszQFi2Bexk1p8LFlbCWvSg\r\n\
//             nTVAixqk9qvTddVzgHbmkWQR0JPazyjTV6+DNQCGcJTGjWV+EcGlB5Z3ELSRCDHG\r\n\
//             5RigFuBDFgCWIxhtZmL0ArwAL8AL8AK8AC/AC/ACvAAvwP8FoE4DmDMAHFV6AKg3\r\n\
//             cOG7aGqW1pqe/lf/lWkqP81IF2z5rKurtdnwtak+oiE7h98I7/8xk1+/SD0A0Goc\r\n\
//             g2o5qr0YgGOq2u4LLgXw8rL1zug6AL4vfyMAD7fc+6ea9wCEK2RtV9pJ5L0AsZdC\r\n\
//             E8rW1h3QuwD4qG3YhudpfznA0EESegfAyO1ZF45fBcDHzhWsB+WKphFJ+OysrxVg\r\n\
//             HK/XdRQqhcfjS8zRCYAkrFPzzIGmLUFj08NBgCG8WsY7Hf5w8XoEwO9oFlJ4HdWX\r\n\
//             bjx0bwfgiID6nDIBmK8dOO8BcG7Z56R572eXHm0A65VMtTHyk/eT2eQBjqg4LY8y\r\n\
//             eQ2SNABwfdb7ZiIvPqZhCrg9F3sKCufHulDoagDO4ZS8pHgmHy6Ba2OAozP6wthC\r\n\
//             fqwLWfUy5JidGXy+7rIGvNCGUgcwtG0dUodfW2Nqfvx9E7jaZHRYX0Hu+uj7zJNQ\r\n\
//             m4yGtq3WefdjLyY+ZJD8F6VL8BqAA/qhofSrHrsrZ22nBlUC6EZ9IVM362ixy9vM\r\n\
//             5nDQrwCgjeMXLHzQO2edN1grVqA+HZ/d/ZeL4YZ0fIOprA3rLwCKLsznAVTZhPo4\r\n\
//             gJl0AT4MkJdDfwDAFjqQHwRgiw3QTwEoIMsdwI8ACFZUBA8DePW8IngYQBmnp+q+\r\n\
//             JMJPAMSsiPlM3ZdEcLcHQl+v1TPxdFLGfC1+01YcnpoZoJaklx4LJ2XprIpcvBUr\r\n\
//             A84SbxpjxOfSw0mZHZ+elR3xV3hASAbJ44X8ryIi+bs3elcuUMH9LvohmSbEUgpl\r\n\
//             RTTpyL6jIhJCBRNi6cmhnHXkP1uSZScjf5ENJVnYEJ/zgKKYH9qKxZ3yTwEwio9n\r\n\
//             w9MAq6XAIwCT7+NnARSjZPcV3C7AsRfMMRdrVPEG72oPhCc28Qn0QlK61gPqu+H6\r\n\
//             v6SU8alrenxdq33cA6HM8zlHDxYuwzninDcpH/eAAM1LO3Ot1g4Qzrr766w5GSl2\r\n\
//             sU0Ob/4BrGxXIweWt2UAAAAASUVORK5CYII=\r\n\
//             =/u+x\r\n\
//             -----END PGP MESSAGE-----
//         ";

//         let armor = Armor::parse_str(armor_str);

//         assert_eq!(armor.data_type, ArmorDataType::PgpMessage);
//         // assert_eq!(
//         //     armor.headers.get(&ArmorHeaderKey::Version),
//         //     Some(&vec![String::from("OpenPrivacy 0.99")])
//         // );
//         // assert_eq!(armor.data, "\
//         //     iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAMAAAD04JH5AAAAM1BMVEUAk90NmN48\r\n\
//         //     reaV0/F2xe0Ane284/f///8AjdsAmOb2+/7W7vro9fwno+Ku3fVStehcuumhrI4O\r\n\
//         //     AAAFcklEQVR42u2b2ZLcIAxFAWEas///14bF7gav4G2qUtZDMnnI3IMQSAaBPidM\r\n\
//         //     KClZMKmEOPg70MH/pxg4ojFCqPeGENaWghQPAUgguu+7LigPFv+JNDXidgAFBAex\r\n\
//         //     Jeu7XlN5K4CkXh1tWN8hx24DUF5+HDsP9vuJ/xg6ZOUtAAL0IO/1MHEUTFwDBoD6\r\n\
//         //     gMwgOgziegBGkrzXIdSomXcMJWhk6DsiLwYQ3vtJPgsz6UdPqXM0esNvCj5Av04w\r\n\
//         //     lwKoNHwvD2pci87vAzwzjImlFn8XJlwIwHQa/ji3ficI2oiXS6CIxb6jlwEYnIbv\r\n\
//         //     0uiZw5yjfashqAKAuOX54afRW1SjXkmA6vW1TMFYLV9FUAFgkj5RKRga5EMgwGkA\r\n\
//         //     hjN9QG36qMfsJIDSyf9BX7hG+UCg1SkAQboYfzLuBe36PgzcKQCa1r9Z0+9TPXI8\r\n\
//         //     DNBeAEZ9GueCz5J/j7U3HH7ccAFWhwFUCkAtwlyU+l2PHTDli0FfGfo8hNcZNtfi\r\n\
//         //     NoD7ToCwvJS3k3woqV5HkAcBWJoAG2KBFxNP2FKthldqpY4cA0grIPIDLyZ1JayU\r\n\
//         //     WwnInh0CgG50gMz3n61aA5YJNlywASB0+mWsDMDOit19s8EFaNcBpAyAbf01gs62\r\n\
//         //     AwwO4PBhhTNFVe6aGpbNAGkPQljlE7C3s/88V7sXrAPYMQTzFdDXVJqkW8pJohEg\r\n\
//         //     bYJ+E1K4ZiqLPQktTMIaOtpxJFZZBPa4rtinXX1SRDszQFi2Bexk1p8LFlbCWvSg\r\n\
//         //     nTVAixqk9qvTddVzgHbmkWQR0JPazyjTV6+DNQCGcJTGjWV+EcGlB5Z3ELSRCDHG\r\n\
//         //     5RigFuBDFgCWIxhtZmL0ArwAL8AL8AK8AC/AC/ACvAAvwP8FoE4DmDMAHFV6AKg3\r\n\
//         //     cOG7aGqW1pqe/lf/lWkqP81IF2z5rKurtdnwtak+oiE7h98I7/8xk1+/SD0A0Goc\r\n\
//         //     g2o5qr0YgGOq2u4LLgXw8rL1zug6AL4vfyMAD7fc+6ea9wCEK2RtV9pJ5L0AsZdC\r\n\
//         //     E8rW1h3QuwD4qG3YhudpfznA0EESegfAyO1ZF45fBcDHzhWsB+WKphFJ+OysrxVg\r\n\
//         //     HK/XdRQqhcfjS8zRCYAkrFPzzIGmLUFj08NBgCG8WsY7Hf5w8XoEwO9oFlJ4HdWX\r\n\
//         //     bjx0bwfgiID6nDIBmK8dOO8BcG7Z56R572eXHm0A65VMtTHyk/eT2eQBjqg4LY8y\r\n\
//         //     eQ2SNABwfdb7ZiIvPqZhCrg9F3sKCufHulDoagDO4ZS8pHgmHy6Ba2OAozP6wthC\r\n\
//         //     fqwLWfUy5JidGXy+7rIGvNCGUgcwtG0dUodfW2Nqfvx9E7jaZHRYX0Hu+uj7zJNQ\r\n\
//         //     m4yGtq3WefdjLyY+ZJD8F6VL8BqAA/qhofSrHrsrZ22nBlUC6EZ9IVM362ixy9vM\r\n\
//         //     5nDQrwCgjeMXLHzQO2edN1grVqA+HZ/d/ZeL4YZ0fIOprA3rLwCKLsznAVTZhPo4\r\n\
//         //     gJl0AT4MkJdDfwDAFjqQHwRgiw3QTwEoIMsdwI8ACFZUBA8DePW8IngYQBmnp+q+\r\n\
//         //     JMJPAMSsiPlM3ZdEcLcHQl+v1TPxdFLGfC1+01YcnpoZoJaklx4LJ2XprIpcvBUr\r\n\
//         //     A84SbxpjxOfSw0mZHZ+elR3xV3hASAbJ44X8ryIi+bs3elcuUMH9LvohmSbEUgpl\r\n\
//         //     RTTpyL6jIhJCBRNi6cmhnHXkP1uSZScjf5ENJVnYEJ/zgKKYH9qKxZ3yTwEwio9n\r\n\
//         //     w9MAq6XAIwCT7+NnARSjZPcV3C7AsRfMMRdrVPEG72oPhCc28Qn0QlK61gPqu+H6\r\n\
//         //     v6SU8alrenxdq33cA6HM8zlHDxYuwzninDcpH/eAAM1LO3Ot1g4Qzrr766w5GSl2\r\n\
//         //     sU0Ob/4BrGxXIweWt2UAAAAASUVORK5CYII=\
//         // ");

//         // assert_eq!(armor.checksum, "=/u+x");
//     }
// }
