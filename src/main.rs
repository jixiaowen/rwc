mod processor;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "rwc",
    version = "0.1.0",
    about = "高性能行计数工具，支持多字符分隔符和多种编码",
    long_about = "类似于 wc -l，但支持自定义多字符分隔符和 GBK/UTF-8 编码"
)]
struct Args {
    /// 输入文件路径（不指定则从标准输入读取）
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// 分隔符（默认为 \n）支持多字符分隔符如 "|+|\n"
    #[arg(short = 'd', long = "delimiter", default_value = "\n")]
    delimiter: String,

    /// 文件编码（utf8, gbk, auto）
    #[arg(short = 'e', long = "encoding", default_value = "auto")]
    encoding: String,

    /// 显示详细信息
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 处理分隔符转义序列
    let delimiter = args.delimiter
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t");

    if args.files.is_empty() {
        // 从标准输入读取
        let count = processor::process_stdin(&delimiter, &args.encoding)?;
        println!("{}", count);
    } else {
        // 处理文件
        for file in &args.files {
            let count = processor::process_file(file, &delimiter, &args.encoding)?;
            if args.files.len() > 1 || args.verbose {
                println!("{} {}", count, file.display());
            } else {
                println!("{}", count);
            }
        }
    }

    Ok(())
}
