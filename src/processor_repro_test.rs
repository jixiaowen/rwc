#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_repro() {
        // This test is designed to trigger the "byte index is not a char boundary" panic.
        // We need a multi-byte character at the end of a chunk, and a short delimiter.
        // "😊" is 4 bytes: [0xF0, 0x9F, 0x98, 0x8A]
        // If delimiter is "AB" (2 bytes), the code tries to keep suffix of length 2-1=1.
        // It slices at len - 1. 
        // If the chunk ends with "😊", len is at the end of the 4th byte.
        // len - 1 is inside the 4th byte (specifically, at the start of the 4th byte, index 3 of the char bytes).
        // Wait, "😊" bytes are [0, 1, 2, 3]. len is 4. len-1 is 3.
        // Byte 3 is 0x8A, which is a continuation byte, not a char boundary.
        
        let delimiter = "AB"; 
        // Encoding doesn't matter much if we construct the string directly, but the function takes a reader.
        // We'll simulate the logic directly or via a Reader.
        // Let's use the public API  or verify logic in a unit test.
        // Since  is private but used by , 
        // we can test  with a temporary file.
    }
}
