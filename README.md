# rwc - Rust Word Counter

高性能的命令行行计数工具，支持多字符分隔符和多种文本编码。

## 特性

✅ **高性能** - 比 Linux `wc` 命令快约 20%（109MB 文件测试：0.086s vs 0.103s）  
✅ **低内存占用** - 使用内存映射技术，大文件处理内存占用稳定（约 2x 文件大小）  
✅ **多编码支持** - 自动检测或手动指定 UTF-8、GBK 等编码  
✅ **多字符分隔符** - 支持任意长度的字符串分隔符（如 `|+|\n`）  
✅ **标准输入支持** - 可从管道读取数据

## 安装

```bash
cargo build --release
```

编译后的可执行文件位于 `target/release/rwc`

## 使用方法

### 基本用法

```bash
# 统计文件行数（默认按 \n 分割）
./target/release/rwc file.txt

# 统计多个文件
./target/release/rwc file1.txt file2.txt file3.txt

# 从标准输入读取
cat file.txt | ./target/release/rwc
echo "测试文本" | ./target/release/rwc
```

### 自定义分隔符

```bash
# 使用多字符分隔符
./target/release/rwc -d "|+|\n" data.txt

# 使用逗号分隔
./target/release/rwc -d "," data.csv

# 使用多个换行符作为段落分隔符
./target/release/rwc -d "\n\n" document.txt
```

### 指定编码

```bash
# 使用 GBK 编码
./target/release/rwc -e gbk chinese_file.txt

# 使用 UTF-8 编码
./target/release/rwc -e utf8 file.txt

# 自动检测编码（默认）
./target/release/rwc -e auto file.txt
```

### 详细输出

```bash
# 显示文件名（处理多个文件时自动显示）
./target/release/rwc -v file.txt
```

## 性能测试

### 测试环境
- 文件大小：109MB
- 行数：10,000,001 行
- 系统：macOS

### 测试结果

| 工具 | 执行时间 | 内存占用 |
|------|---------|---------|
| **rwc** | **0.086s** | **229MB** |
| wc -l | 0.103s | - |

**结论**：rwc 比 wc 快约 **20%**

### 功能测试

✅ 默认换行符分割：通过  
✅ 多字符分隔符（`|+|\n`）：通过  
✅ UTF-8 中文文本：通过  
✅ 标准输入处理：通过  
✅ 大文件处理（109MB）：通过  
✅ 单元测试（6个）：全部通过

## 命令行参数

```
Usage: rwc [OPTIONS] [FILE]...

Arguments:
  [FILE]...
          输入文件路径（不指定则从标准输入读取）

Options:
  -d, --delimiter <DELIMITER>
          分隔符（默认为 \n）支持多字符分隔符如 "|+|\n"
          [default: "\n"]

  -e, --encoding <ENCODING>
          文件编码（utf8, gbk, auto）
          [default: auto]

  -v, --verbose
          显示详细信息

  -h, --help
          显示帮助信息

  -V, --version
          显示版本信息
```

## 技术实现

- **语言**: Rust
- **内存映射**: 使用 `memmap2` 处理大文件
- **编码转换**: 使用 `encoding_rs` 高效转换编码
- **CLI 解析**: 使用 `clap` v4 的 derive 宏
- **优化级别**: Release 模式开启 LTO 和最高优化

## 项目结构

```
rwc/
├── Cargo.toml           # 项目配置和依赖
├── src/
│   ├── main.rs          # CLI 入口和参数解析
│   ├── encoding.rs      # 编码检测和转换
│   └── processor.rs     # 文件处理和分隔符计数
└── test_files/          # 测试文件目录
```

## 开发

### 运行测试

```bash
cargo test
```

### 运行基准测试

```bash
# 创建测试文件
seq 1 10000000 > test_files/large_file.txt

# 测试 rwc
time ./target/release/rwc test_files/large_file.txt

# 对比 wc
time wc -l test_files/large_file.txt
```

## 许可证

MIT
