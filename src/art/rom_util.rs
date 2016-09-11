const BIT_REVS: &'static [u8] =
    &[0, 128, 64, 192, 32, 160, 96, 224, 16, 144, 80, 208, 48, 176, 112, 240, 8, 136, 72, 200, 40,
      168, 104, 232, 24, 152, 88, 216, 56, 184, 120, 248, 4, 132, 68, 196, 36, 164, 100, 228, 20,
      148, 84, 212, 52, 180, 116, 244, 12, 140, 76, 204, 44, 172, 108, 236, 28, 156, 92, 220, 60,
      188, 124, 252, 2, 130, 66, 194, 34, 162, 98, 226, 18, 146, 82, 210, 50, 178, 114, 242, 10,
      138, 74, 202, 42, 170, 106, 234, 26, 154, 90, 218, 58, 186, 122, 250, 6, 134, 70, 198, 38,
      166, 102, 230, 22, 150, 86, 214, 54, 182, 118, 246, 14, 142, 78, 206, 46, 174, 110, 238, 30,
      158, 94, 222, 62, 190, 126, 254, 1, 129, 65, 193, 33, 161, 97, 225, 17, 145, 81, 209, 49,
      177, 113, 241, 9, 137, 73, 201, 41, 169, 105, 233, 25, 153, 89, 217, 57, 185, 121, 249, 5,
      133, 69, 197, 37, 165, 101, 229, 21, 149, 85, 213, 53, 181, 117, 245, 13, 141, 77, 205, 45,
      173, 109, 237, 29, 157, 93, 221, 61, 189, 125, 253, 3, 131, 67, 195, 35, 163, 99, 227, 19,
      147, 83, 211, 51, 179, 115, 243, 11, 139, 75, 203, 43, 171, 107, 235, 27, 155, 91, 219, 59,
      187, 123, 251, 7, 135, 71, 199, 39, 167, 103, 231, 23, 151, 87, 215, 55, 183, 119, 247, 15,
      143, 79, 207, 47, 175, 111, 239, 31, 159, 95, 223, 63, 191, 127, 255];

pub fn reverse_byte(v: u8) -> u8 {
    BIT_REVS[(v & 0xFF) as usize]
}

pub fn decompress(rom: &[u8], address: u32, max_length: u32) -> Result<Vec<u8>, String> {
    let input = &rom[address as usize..];
    let mut output = Vec::new();

    let mut opos = 0;

    let mut i = input.iter();
    let mut v = *i.next().unwrap();//.nth(address as usize).unwrap();
    while v != 0xFF {
        // ccclllll
        let mut cmdtype = v as u16 >> 5;
        let mut len = (v as u16 & 0x1F) + 1;

        // Get 2 bits for length
        if cmdtype == 7 {
            // 111cccll llllllll
            cmdtype = (v as u16 & 0x1C) >> 2;
            let a = v;
            v = *i.next().unwrap();
            len = ((a as u16 & 3) << 8) + v as u16 + 1;
        }

        if output.len() + len as usize > max_length as usize {
            return Err("Output greater than max_length".to_string());
        }

        v = *i.next().unwrap();

        // Get output read location
        if cmdtype >= 4 {
            let a = v as u16;
            v = *i.next().unwrap();
            opos = (a << 8) + v as u16;
            v = *i.next().unwrap();
        }

        match cmdtype {
            0 => {
                // Copy
                for _ in 0..len {
                    output.push(v);
                    v = *i.next().unwrap();
                }
            }
            1 => {
                // RLE
                for _ in 0..len {
                    output.push(v);
                }
                v = *i.next().unwrap();
            }
            2 => {
                // 2 byte RLE
                let a = [v, *i.next().unwrap()];
                for _ in 0..len {
                    output.extend(a.iter());
                }
                v = *i.next().unwrap();
            }
            3 => {
                // Incrementing value
                let mut a = v as u8;
                for _ in 0..len {
                    output.push(a);
                    a += 1;
                }
                v = *i.next().unwrap();
            }
            4 => {
                // Read from output
                for p in 0..len {
                    let a = output[(opos + p as u16) as usize];
                    output.push(a);
                }
            }
            5 => {
                // Read from output reversed
                for p in 0..len {
                    let a = output[(opos + p as u16) as usize] & 0xFF;
                    output.push(reverse_byte(a));
                }
            }
            6 => {
                // Read from output backwards
                for p in 0..len {
                    let a = output[(opos - p as u16) as usize];
                    output.push(a);
                }
            }
            7 => return Err("Command 7".to_string()),
            _ => return Err("Unknown command!".to_string()),
        }
    }

    Ok(output)
}


// pub fn to_snes(address: i32) -> i32 {
//   address + 0xC00000 - 0x200
// }
pub fn from_snes(address: i32) -> i32 {
    address - 0xC00000 + 0x200
}
