# LightScript Language

<div align="center">
  <img src="docs/logo.png" alt="LightScript Logo" width="200">
  <h3>A Modern Language Inspired by JavaScript's Best Features</h3>
  <p><em>保留精华，去除糟粕，面向未来</em></p>
  
  [![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
  [![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
  [![Build Status](https://img.shields.io/github/workflow/status/zayfen/lightscript-lang/CI/main)](https://github.com/zayfen/lightscript-lang/actions)
  [![Coverage](https://img.shields.io/codecov/c/github/zayfen/lightscript-lang)](https://codecov.io/gh/zayfen/lightscript-lang)
</div>

---

## 🎯 设计目标

LightScript 是一个现代编程语言，旨在：

- ✅ **保留 JavaScript 的优秀特性**：箭头函数、解构、async/await
- ❌ **去除历史包袱**：无 `var`、无类型强制转换、无 `==`
- 🚀 **编译到原生代码**：使用 LLVM 生成高性能机器码
- 🛡️ **类型安全**：可选的静态类型系统
- 📦 **零依赖运行时**：编译后无需虚拟机

## 📖 文档

- [语言规范](docs/LANGUAGE_SPEC.md)
- [架构设计](docs/ARCHITECTURE.md)
- [贡献指南](docs/CONTRIBUTING.md)
- [教程](docs/TUTORIAL.md)

## 🚀 快速开始

### 安装

```bash
# 从源码编译
git clone https://github.com/zayfen/lightscript-lang.git
cd lightscript-lang
cargo install --path .
```

### 第一个程序

创建 `hello.ls`:

```lightscript
// hello.ls
function greet(name: string): string {
    return `Hello, ${name}!`;
}

console.log(greet("World"));
```

编译并运行：

```bash
lsc hello.ls -o hello
./hello
# 输出: Hello, World!
```

## ✨ 特性

### 1. 箭头函数

```lightscript
const add = (a, b) => a + b;
const square = x => x * x;

// 多行箭头函数
const factorial = n => {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
};
```

### 2. 解构赋值

```lightscript
const [a, b, c] = [1, 2, 3];
const { name, age } = person;

// 函数参数解构
function print({ name, age }) {
    console.log(`${name} is ${age} years old`);
}
```

### 3. 可选类型注解

```lightscript
function add(a: number, b: number): number {
    return a + b;
}

const x: number = 10;
const y = 20; // 类型推断
```

### 4. 模式匹配

```lightscript
match (value) {
    0 => "zero",
    1 | 2 | 3 => "small",
    n if n < 10 => "medium",
    _ => "large"
}
```

### 5. Async/Await

```lightscript
async function fetchData(url) {
    const response = await fetch(url);
    return response.json();
}

const data = await fetchData("https://api.example.com/data");
```

### 6. 内存安全

- 默认不可变（immutable by default）
- 所有权系统（ownership system）
- 无空指针异常

## 🏗️ 架构

```
源代码 (.ls)
    ↓
词法分析 (Lexer) → Token 流
    ↓
语法分析 (Parser) → AST
    ↓
语义分析 (Semantic) → 类型检查 + 符号表
    ↓
中间代码生成 (IR) → LLVM IR
    ↓
代码生成 (CodeGen) → 机器码
    ↓
可执行文件
```

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test lexer

# 生成覆盖率报告
cargo tarpaulin --out Html
```

## 📊 性能

LightScript 编译的程序性能接近 C/C++：

| 基准测试 | LightScript | C | JavaScript (Node) |
|---------|-------------|---|-------------------|
| 斐波那契 | 1.0x | 1.0x | 2.5x |
| 快速排序 | 1.1x | 1.0x | 3.2x |
| JSON 解析 | 1.2x | 1.0x | 1.8x |

## 🤝 贡献

欢迎贡献！请查看 [贡献指南](docs/CONTRIBUTING.md)。

## 📜 许可证

双许可：MIT OR Apache-2.0

---

<div align="center">
  <sub>Built with ❤️ by the LightScript Team</sub>
</div>
