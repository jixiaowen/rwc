use anyhow::Result;
use memmap2::Mmap;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use crate::encoding;

/// 计算给定文本中分隔符的数量
fn count_delimiters(text: &str, delimiter: &str) -> usize {
    if delimiter.is_empty() {
        return 0;
    }

    // 对于单字符分隔符，使用更快的方法
    if delimiter.len() == 1 {
        let delim_char = delimiter.chars().next().unwrap();
        return text.chars().filter(|&c| c == delim_char).count();
    }

    // 对于多字符分隔符，使用 matches()
    text.matches(delimiter).count()
}

/// 处理文件并返回行数
pub fn process_file(path: &Path, delimiter: &str, encoding_hint: &str) -> Result<usize> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    
    // 对于大文件使用内存映射以提高性能
    let count = if metadata.len() > 1024 * 1024 {
        // 使用内存映射
        let mmap = unsafe { Mmap::map(&file)? };
        let text = encoding::decode_bytes(&mmap[..], encoding_hint)?;
        count_delimiters(&text, delimiter)
    } else {
        // 小文件直接读取
        let mut buffer = Vec::new();
        let mut file = file;
        file.read_to_end(&mut buffer)?;
        let text = encoding::decode_bytes(&buffer, encoding_hint)?;
        count_delimiters(&text, delimiter)
    };

    Ok(count)
}

/// 从标准输入处理并返回行数
pub fn process_stdin(delimiter: &str, encoding_hint: &str) -> Result<usize> {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)?;
    
    let text = encoding::decode_bytes(&buffer, encoding_hint)?;
    Ok(count_delimiters(&text, delimiter))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_single_char_delimiter() {
        let text = "line1\nline2\nline3\n";
        assert_eq!(count_delimiters(text, "\n"), 3);
    }

    #[test]
    fn test_count_multi_char_delimiter() {
        let text = "part1|+|part2|+|part3";
        assert_eq!(count_delimiters(text, "|+|"), 2);
    }

    #[test]
    fn test_empty_delimiter() {
        let text = "some text";
        assert_eq!(count_delimiters(text, ""), 0);
    }

    #[test]
    fn test_no_matches() {
        let text = "no newlines here";
        assert_eq!(count_delimiters(text, "\n"), 0);
    }
}
