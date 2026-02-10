use anyhow::{anyhow, Result};
use encoding_rs::{Encoding, GBK, UTF_8};

/// 检测文件编码
pub fn detect_encoding(bytes: &[u8]) -> &'static Encoding {
    // 检查 UTF-8 BOM
    if bytes.len() >= 3 && &bytes[0..3] == b"\xEF\xBB\xBF" {
        return UTF_8;
    }

    // 尝试验证是否为有效的 UTF-8
    if std::str::from_utf8(bytes).is_ok() {
        return UTF_8;
    }

    // 默认假设为 GBK（可以扩展更复杂的检测逻辑）
    GBK
}

/// 根据指定编码或自动检测将字节转换为字符串
pub fn decode_bytes(bytes: &[u8], encoding_hint: &str) -> Result<String> {
    let encoding = match encoding_hint.to_lowercase().as_str() {
        "utf8" | "utf-8" => UTF_8,
        "gbk" => GBK,
        "auto" => detect_encoding(bytes),
        _ => return Err(anyhow!("不支持的编码: {}", encoding_hint)),
    };

    let (result, _, had_errors) = encoding.decode(bytes);
    
    if had_errors {
        eprintln!("警告: 解码过程中发现错误，可能存在编码问题");
    }

    Ok(result.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_utf8() {
        let utf8_bytes = "你好世界".as_bytes();
        assert_eq!(detect_encoding(utf8_bytes), UTF_8);
    }

    #[test]
    fn test_decode_utf8() {
        let bytes = "Hello\nWorld\n你好".as_bytes();
        let result = decode_bytes(bytes, "utf8").unwrap();
        assert_eq!(result, "Hello\nWorld\n你好");
    }
}
