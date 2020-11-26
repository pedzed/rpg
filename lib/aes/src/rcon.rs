pub fn rcon(round_number: usize) -> u8 {
    let mut rcon: i16 = 0x8D;

    for _ in 0..round_number {
        // rcon = ((rcon << 1) ^ (0x11B & -(rcon >> 7))) & 0xFF;
        rcon = (rcon << 1) ^ (0x11B & -(rcon >> 7));
    }

    rcon as u8
}

#[cfg(test)]
mod tests {
    #[test]
    fn rcon() {
        let rcon_list: [u8; 11] =
            [0x8D, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1B, 0x36];

        for (i, _) in rcon_list.iter().enumerate() {
            assert_eq!(super::rcon(i), rcon_list[i]);
        }
    }
}
