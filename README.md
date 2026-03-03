# LightLang - 系统级编程语言

<div align="center">
  <img src="docs/logo.png" alt="LightLang Logo" width="200">
  <h3>A Modern Systems Programming Language</h3>
  <p><em>安全、高效、现代化的系统级编程语言</em></p>
  
  [![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
  [![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
  [![Build Status](https://img.shields.io/github/workflow/status/zayfen/lightlang/CI/main)](https://github.com/zayfen/lightlang/actions)
  [![Coverage](https://img.shields.io/codecov/c/github/zayfen/lightlang)](https://codecov.io/gh/zayfen/lightlang)
</div>

---

## 🎯 设计目标

LightLang 是一个**系统级编程语言**，设计目标：

- 🦀 **内存安全**：所有权系统 + 借用检查器
- ⚡ **零成本抽象**：编译时优化，无运行时开销
- 🎯 **生成原生代码**：直接编译为 ELF 可执行文件
- 🔒 **类型安全**：强类型系统，编译时捕获错误
- 📦 **零依赖运行时**：编译后无需任何运行时库
- 🛠️ **系统编程**：适合操作系统、驱动、嵌入式开发

## 🆚 对比其他语言

| 特性 | LightLang | Rust | C | C++ |
|------|-----------|------|---|-----|
| 内存安全 | ✅ | ✅ | ❌ | ❌ |
| 零成本抽象 | ✅ | ✅ | ❌ | ✅ |
| 编译速度 | 🚀 | 慢 | 快 | 慢 |
| 学习曲线 | 中等 | 陡峭 | 简单 | 复杂 |
| 运行时 | 无 | 无 | 无 | 有（可选）|
| 输出格式 | ELF | ELF | ELF | ELF |

## 📖 文档

- [语言规范](docs/LANGUAGE_SPEC.md)
- [架构设计](docs/ARCHITECTURE.md)
- [贡献指南](docs/CONTRIBUTING.md)
- [教程](docs/TUTORIAL.md)

## 🚀 快速开始

### 安装

```bash
# 从源码编译
git clone https://github.com/zayfen/lightlang.git
cd lightlang
cargo install --path .
```

### 第一个程序

创建 `hello.ll`:

```lightlang
// hello.ll - 系统级编程示例
use std::io;

fn main(): i32 {
    io::println("Hello, World!");
    return 0;
}
```

编译并运行：

```bash
llc hello.ll -o hello
./hello
# 输出: Hello, World!
```

查看生成的 ELF 文件：

```bash
file hello
# 输出: hello: ELF 64-bit LSB executable, x86-64

readelf -h hello
# 查看ELF头信息
```

## ✨ 核心特性

### 1. 所有权系统

```lightlang
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1 所有权转移给 s2
    
    // println(s1);  // ❌ 编译错误：s1 已失效
    println(s2);     // ✅ 正确
}
```

### 2. 借用和引用

```lightlang
fn len(s: &String): usize {
    return s.len();
}

fn main() {
    let s = String::from("hello");
    let length = len(&s);  // 借用，不转移所有权
    println(s);  // ✅ s 仍然有效
}
```

### 3. 零成本抽象

```lightlang
// 泛型 - 编译时单态化
fn add<T: Add>(a: T, b: T): T {
    return a + b;
}

// 编译后生成特定类型的高效代码
let x = add(1, 2);      // 生成 add_i64
let y = add(1.0, 2.0);  // 生成 add_f64
```

### 4. 内存布局控制

```lightlang
// C 兼容的结构体
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}

// 指定对齐
#[repr(align(16))]
struct AlignedBuffer {
    data: [u8; 256],
}
```

### 5. 内联汇编

```lightlang
fn get_cycles(): u64 {
    let mut cycles: u64;
    unsafe {
        asm!(
            "rdtsc",
            "shl rdx, 32",
            "or rax, rdx",
            out("rax") cycles,
        );
    }
    return cycles;
}
```

### 6. 系统调用

```lightlang
// 直接系统调用
fn sys_write(fd: i32, buf: *const u8, count: usize): isize {
    return syscall(1, fd, buf, count);
}

// 内存映射
fn mmap(addr: *mut void, len: usize, prot: i32, flags: i32): *mut void {
    return syscall(9, addr, len, prot, flags, -1, 0);
}
```

## 🏗️ 编译流程

```
源代码 (.ll)
    ↓
词法分析 → Token 流
    ↓
语法分析 → AST
    ↓
语义分析 → 类型检查 + 借用检查
    ↓
中间代码生成 → LLVM IR
    ↓
优化 → 优化的 LLVM IR
    ↓
代码生成 → 汇编代码 (.s)
    ↓
汇编 → 目标文件 (.o)
    ↓
链接 → ELF 可执行文件
```

## 🔧 编译选项

```bash
# 编译为 ELF 可执行文件
llc source.ll -o program

# 编译为目标文件（不链接）
llc source.ll -c -o program.o

# 生成汇编代码
llc source.ll -S -o program.s

# 生成 LLVM IR
llc source.ll --emit-llvm -o program.ll

# 优化级别
llc source.ll -O0  # 无优化
llc source.ll -O1  # 基本优化
llc source.ll -O2  # 标准优化
llc source.ll -O3  # 激进优化
llc source.ll -Os  # 优化大小

# 目标架构
llc source.ll --target x86_64-linux-gnu
llc source.ll --target aarch64-linux-gnu
llc source.ll --target riscv64-linux-gnu
```

## 📊 性能

LightLang 编译的程序性能接近 C：

| 基准测试 | LightLang | C | Rust |
|---------|-----------|---|------|
| 二分查找 | 1.0x | 1.0x | 1.0x |
| 快速排序 | 1.02x | 1.0x | 0.98x |
| 矩阵乘法 | 0.99x | 1.0x | 1.01x |
| 文件 I/O | 1.0x | 1.0x | 1.0x |

## 🛠️ 工具链

### 编译器 (llc)
- 词法分析
- 语法分析
- 类型检查
- 借用检查
- LLVM IR 生成
- ELF 链接

### 包管理器 (llpm)
- 依赖管理
- 构建系统
- 发布包

### 语言服务器 (lls)
- IDE 集成
- 代码补全
- 类型提示
- 错误诊断

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test lexer

# 生成覆盖率报告
cargo tarpaulin --out Html
```

## 📚 标准库

### 核心类型
- `i8`, `i16`, `i32`, `i64`, `i128` - 有符号整数
- `u8`, `u16`, `u32`, `u64`, `u128` - 无符号整数
- `f32`, `f64` - 浮点数
- `bool` - 布尔值
- `char` - Unicode 字符
- `String` - 字符串
- `Vec<T>` - 动态数组
- `HashMap<K, V>` - 哈希表

### 系统接口
- `std::io` - 输入输出
- `std::fs` - 文件系统
- `std::net` - 网络
- `std::thread` - 线程
- `std::sync` - 同步原语
- `std::mem` - 内存操作
- `std::ptr` - 指针操作

## 🤝 贡献

欢迎贡献！请查看 [贡献指南](docs/CONTRIBUTING.md)。

## 📜 许可证

双许可：MIT OR Apache-2.0

---

<div align="center">
  <sub>Built with ❤️ | Powered by LLVM | Generates Native ELF Binaries</sub>
</div>
