
use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{Cursor, Read};

const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664; // x64
const IMAGE_FILE_MACHINE_ARM: u16 = 0x1c0; // ARM little endian
const IMAGE_FILE_MACHINE_ARMNT: u16 = 0x1c4; // ARM Thumb-2 little endian
const IMAGE_FILE_MACHINE_ARM64: u16 = 0xaa64; // ARM64 little endian
const IMAGE_FILE_MACHINE_I386: u16 = 0x14c; // Intel 386 or later processors and compatible processors

lazy_static! {
    static ref PATCHES: HashMap<&'static str, Vec<(Regex, String, i32)>> = {
        let mut patches = HashMap::new();
        pub fn cr(pattern: &'static str) -> String { pattern.replace(" ", "").to_lowercase() }
        let mr = |pattern: &'static str| Regex::new(&cr(pattern)).unwrap();
        patches.insert("amd64", vec![
            (
                mr("(39 9E C8 00 00 00) 0F 95 C1 (88 0F 8B)"),
                cr("${1} B1 00 90 ${2}"),
                0
            ),
            (
                mr(r#"(FF EB 05) 8A 49 61 (88 0A 8B CB E8)"#),
                cr(r#"${1} B1 00 90 ${2}"#),
                0
            )
        ]);
        patches.insert("i386", vec![
            (
                mr(r#"(FF EB 08 39 77 74) 0F 95 C1 (88 08 8B)"#),
                cr(r#"${1} B1 00 90 ${2}"#),
                0
            ),
            (
                mr(r#"(FF EB 08 8B 4D 08) 8A 49 31 (88 08 8B)"#),
                cr(r#"${1} B1 00 90 ${2}"#),
                0
            )
        ]);
        patches.insert("arm", vec![
            (
                mr(r#"(05 E0 .. 3 .. 0B) B1 01 (23 00 E0 00 23 2B 70 20 46)"#),
                cr(r#"${1} B1 00 ${2}"#),
                0
            ),
            (
                mr(r#"(02 E0) 90 F8 .. 30 (0B 70 20 46)"#),
                cr(r#"${1} 4F F0 00 03 ${2}"#),
                0
            )
        ]);
        patches.insert("arm64", vec![
            (
                mr(r#"(FE 97 05 00 00 14 A8 .. A 40 B9 1F 01 00 71) E9 07 9F 1A (89 02 00 39 E0 03 13 2A)"#),
                cr(r#"${1} 09 00 80 52 ${2}"#),
                0
            ),
            (
                mr(r#"(FC 97 03 00 00 14 08) .. 41 39 (28 00 00 39 E0 03 13 2A)"#),
                cr(r#"${1} 00 80 52 ${2}"#),
                1
            )
        ]);

        patches
    };
}

pub fn check_machine(data: &[u8]) -> Result<String, String> {
    // Create a cursor to read from the bytes
    let mut cursor = Cursor::new(data);

    // Seek to the COFF header offset
    cursor.set_position(0x3C);
    let mut coff_offset_bytes = [0; 4];
    cursor.read_exact(&mut coff_offset_bytes)
          .map_err(|e| format!("Error reading COFF header offset: {}", e))?;
    let coff_offset = u32::from_le_bytes(coff_offset_bytes) as u64;

    // Seek to the COFF header
    cursor.set_position(coff_offset);
    cursor.set_position(cursor.position() + 4); // Skip signature

    // Read machine header
    let mut machine_bytes = [0; 2];
    cursor.read_exact(&mut machine_bytes)
          .map_err(|e| format!("Error reading machine header: {}", e))?;
    let machine = u16::from_le_bytes(machine_bytes);

    // Determine architecture based on machine header
    match machine {
        IMAGE_FILE_MACHINE_AMD64 => Ok(String::from("amd64")),
        IMAGE_FILE_MACHINE_I386 => Ok(String::from("i386")),
        IMAGE_FILE_MACHINE_ARM | IMAGE_FILE_MACHINE_ARMNT => Ok(String::from("arm")),
        IMAGE_FILE_MACHINE_ARM64 => Ok(String::from("arm64")),
        _ => Err(format!("Unsupported machine header: {}", machine)),
    }
}

pub fn patch_module(architecture: &str, dll_data: &[u8]) -> Result<Vec<u8>, String> {
    let dll_data_hex = hex::encode(dll_data);
    let mut patched_data = dll_data_hex.clone();
    if let Some(patches_for_arch) = PATCHES.get(architecture) {
        for (pattern, replace, count) in patches_for_arch {
            patched_data = pattern.replacen(&patched_data, *count as usize, replace).into_owned();
        }
    } else {
        return Err(format!("Unsupported architecture {}", architecture));
    }
    Ok(hex::decode(patched_data).unwrap())
}
