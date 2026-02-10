use anyhow::Result;
use encoding_rs::{Encoding, GBK, UTF_8};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

const BUFFER_SIZE: usize = 64 * 1024; // 64KB buffer

/// 检测文件编码（从前几KB数据）
fn detect_encoding_from_sample(sample: &[u8]) -> &'static Encoding {
    // 检查 UTF-8 BOM
    if sample.len() >= 3 && &sample[0..3] == b"\xEF\xBB\xBF" {
        return UTF_8;
    }

    // 尝试验证是否为有效的 UTF-8
    if std::str::from_utf8(sample).is_ok() {
        return UTF_8;
    }

    // 默认假设为 GBK
    GBK
}

/// 快速统计单字节分隔符（直接在字节级别，无需解码）
fn count_single_byte_delimiter(file: File, delimiter_byte: u8) -> Result<usize> {
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut delimiter_count = 0;
    let mut has_content = false;

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        has_content = true;
        delimiter_count += memchr::memchr_iter(delimiter_byte, &buffer[..n]).count();
    }

    // 空文件返回0，否则返回段数（分隔符数+1）
    if !has_content {
        Ok(0)
    } else {
        Ok(delimiter_count + 1)
    }
}

/// 统计单字节分隔符（从stdin）
fn count_single_byte_delimiter_stdin(delimiter_byte: u8) -> Result<usize> {
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut delimiter_count = 0;
    let mut has_content = false;
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    loop {
        let n = handle.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        has_content = true;
        delimiter_count += memchr::memchr_iter(delimiter_byte, &buffer[..n]).count();
    }

    // 空输入返回0，否则返回段数（分隔符数+1）
    if !has_content {
        Ok(0)
    } else {
        Ok(delimiter_count + 1)
    }
}

/// 流式统计多字节分隔符
fn count_multi_byte_delimiter_streaming(
    mut reader: BufReader<File>,
    delimiter: &str,
    encoding: &'static Encoding,
) -> Result<usize> {
    let mut decoder = encoding.new_decoder();
    let mut input_buffer = vec![0u8; BUFFER_SIZE];
    let mut output_buffer = String::with_capacity(BUFFER_SIZE * 2);
    let mut leftover = String::new();
    let mut delimiter_count = 0;
    let mut has_content = false;
    let delimiter_len = delimiter.len();

    loop {
        let n = reader.read(&mut input_buffer)?;
        if n == 0 {
            // 处理剩余数据
            if !leftover.is_empty() {
                delimiter_count += leftover.matches(delimiter).count();
            }
            break;
        }

        has_content = true;

        // 解码当前块
        output_buffer.clear();
        let (_result, _bytes_read, _had_errors) = decoder.decode_to_string(
            &input_buffer[..n],
            &mut output_buffer,
            n == 0,
        );

        // 将上次的剩余部分和当前块合并
        let combined = if leftover.is_empty() {
            output_buffer.clone()
        } else {
            let mut temp = leftover.clone();
            temp.push_str(&output_buffer);
            temp
        };

        // 统计完整匹配的分隔符
        delimiter_count += combined.matches(delimiter).count();

        // 保留末尾可能不完整的部分
        leftover.clear();
        if combined.len() >= delimiter_len - 1 {
            let start = combined.len().saturating_sub(delimiter_len - 1);
            leftover.push_str(&combined[start..]);
        }
    }

    // 空文件返回0，否则返回段数（分隔符数+1）
    if !has_content {
        Ok(0)
    } else {
        Ok(delimiter_count + 1)
    }
}

/// 流式统计多字节分隔符（从stdin）
fn count_multi_byte_delimiter_stdin(delimiter: &str, encoding: &'static Encoding) -> Result<usize> {
    let stdin = io::stdin();
    let handle = stdin.lock();
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, handle);
    
    let mut decoder = encoding.new_decoder();
    let mut input_buffer = vec![0u8; BUFFER_SIZE];
    let mut output_buffer = String::with_capacity(BUFFER_SIZE * 2);
    let mut leftover = String::new();
    let mut delimiter_count = 0;
    let mut has_content = false;
    let delimiter_len = delimiter.len();

    loop {
        let n = reader.read(&mut input_buffer)?;
        if n == 0 {
            if !leftover.is_empty() {
                delimiter_count += leftover.matches(delimiter).count();
            }
            break;
        }

        has_content = true;

        output_buffer.clear();
        let (_result, _bytes_read, _had_errors) = decoder.decode_to_string(
            &input_buffer[..n],
            &mut output_buffer,
            false,
        );

        let combined = if leftover.is_empty() {
            output_buffer.clone()
        } else {
            let mut temp = leftover.clone();
            temp.push_str(&output_buffer);
            temp
        };

        delimiter_count += combined.matches(delimiter).count();

        leftover.clear();
        if combined.len() >= delimiter_len - 1 {
            let start = combined.len().saturating_sub(delimiter_len - 1);
            leftover.push_str(&combined[start..]);
        }
    }

    // 空输入返回0，否则返回段数（分隔符数+1）
    if !has_content {
        Ok(0)
    } else {
        Ok(delimiter_count + 1)
    }
}

/// 处理文件并返回分隔符计数
pub fn process_file(path: &Path, delimiter: &str, encoding_hint: &str) -> Result<usize> {
    let file = File::open(path)?;

    // 优化：对于UTF-8文件的单字节分隔符，直接在字节级别处理
    if delimiter.len() == 1 
        && delimiter.is_ascii() 
        && (encoding_hint == "utf8" || encoding_hint == "utf-8" || encoding_hint == "auto")
    {
        let delimiter_byte = delimiter.as_bytes()[0];
        return count_single_byte_delimiter(file, delimiter_byte);
    }

    // 确定编码
    let encoding = match encoding_hint.to_lowercase().as_str() {
        "utf8" | "utf-8" => UTF_8,
        "gbk" => GBK,
        "auto" => {
            // 读取前几KB来检测编码
            let mut sample = vec![0u8; 8192];
            let mut temp_file = File::open(path)?;
            let n = temp_file.read(&mut sample)?;
            detect_encoding_from_sample(&sample[..n])
        }
        _ => UTF_8,
    };

    // 重新打开文件进行流式处理
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(BUFFER_SIZE, file);

    count_multi_byte_delimiter_streaming(reader, delimiter, encoding)
}

/// 从标准输入处理并返回分隔符计数
pub fn process_stdin(delimiter: &str, encoding_hint: &str) -> Result<usize> {
    // 优化：单字节分隔符直接在字节级别处理
    if delimiter.len() == 1 && delimiter.is_ascii() {
        let delimiter_byte = delimiter.as_bytes()[0];
        return count_single_byte_delimiter_stdin(delimiter_byte);
    }

    // 确定编码
    let encoding = match encoding_hint.to_lowercase().as_str() {
        "utf8" | "utf-8" => UTF_8,
        "gbk" => GBK,
        "auto" => UTF_8, // stdin 默认 UTF-8
        _ => UTF_8,
    };

    count_multi_byte_delimiter_stdin(delimiter, encoding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_count_newlines() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "line1")?;
        writeln!(file, "line2")?;
        writeln!(file, "line3")?;
        file.flush()?;

        let count = process_file(file.path(), "\n", "utf8")?;
        assert_eq!(count, 3);
        Ok(())
    }
}
