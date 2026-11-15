use sha2::{Digest, Sha256};

pub fn secret() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("Windows randomness unavailable");
    hex(&bytes)
}
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn hash(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
pub fn equal(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0, |v, (a, b)| v | (a ^ b)) == 0
}

#[cfg(windows)]
pub fn protect(bytes: &[u8], decrypt: bool) -> Result<Vec<u8>, String> {
    use windows::Win32::{
        Foundation::{HLOCAL, LocalFree},
        Security::Cryptography::{CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData},
    };
    let input = CRYPT_INTEGER_BLOB {
        cbData: bytes.len().try_into().map_err(|_| "Secret too large")?,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    // SAFETY: input is a live immutable byte slice, which both DPAPI functions only read. Output is
    // initialized storage; any successful allocation is copied and freed with LocalFree below.
    let result = unsafe {
        if decrypt {
            CryptUnprotectData(&input, None, None, None, None, CRYPTPROTECT_UI_FORBIDDEN, &mut output)
        } else {
            CryptProtectData(
                &input,
                windows::core::PCWSTR::null(),
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        }
    };
    let data = if result.is_ok() && !output.pbData.is_null() {
        // SAFETY: successful DPAPI output contains cbData bytes until LocalFree.
        Some(unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec())
    } else {
        None
    };
    if !output.pbData.is_null() {
        // SAFETY: DPAPI allocated this output with LocalAlloc and ownership has not escaped.
        let _ = unsafe { LocalFree(Some(HLOCAL(output.pbData.cast()))) };
    }
    result.map_err(|_| "Cannot unlock this PC's phone identity for the current Windows user".to_owned())?;
    data.ok_or("Empty DPAPI result".into())
}
#[cfg(not(windows))]
pub fn protect(_: &[u8], _: bool) -> Result<Vec<u8>, String> {
    Err("Windows DPAPI is required for persisted identity".into())
}
