# Ziv 标准库介绍与使用指南

本文档讲解标准库的设计、使用方式、链接路径与当前运行时能力。

## 1. 标准库在哪里

标准库定义位于独立 crate：

- `crates/ziv-stdlib/`

编译器通过 `Stdlib::new()` 读取内置函数元信息（函数名、参数、返回类型、分类、描述）。

## 2. 标准库如何进入编译流程

### 2.1 语义阶段（符号注册）

语义分析器启动时会把标准库函数注册到符号表，后续源码中可直接调用。

效果：

- `print/println/abs/...` 在语义检查阶段被识别为已定义函数
- 参数和返回类型用于基础类型检查

### 2.2 IR 阶段（调用降级）

- `print/println` 会被降级为运行时 helper 调用（例如 `ziv_println_i64`）
- `container` 模块函数（`vector*` / `hashMap*`）会降级为直接 runtime 调用
- 其他标准库函数当前主要用于“可解析/可检查/可编译”能力，后续可接宿主实现

### 2.3 链接阶段（可执行文件）

编译器会生成并链接一个运行时对象（包含输出 helper），保证基本输出可运行。

## 3. 标准库分类

当前共 117 个函数，分为 10 类：

- `io`（9）
- `math`（8）
- `string`（8）
- `array`（8）
- `container`（20）
- `js`（18）
- `filesystem`（12）
- `net`（10）
- `crypto`（12）
- `encoding`（12）

完整签名见：[STDLIB_API.md](STDLIB_API.md)

## 4. 使用方式

### 4.1 直接调用

```ziv
println("io demo");
abs(-10);
strlen("abc");
```

### 4.2 与结构体、函数混用

```ziv
struct User {
    age: int;
    score: int;
}

function show(u: User): int {
    println(u.age);
    return u.score;
}

let u: User = User.(age = 18, score = 90);
println(show(u));
```

### 4.3 函数参数传函数

```ziv
function inc(x: int): int { return x + 1; }
function apply(f: function, v: int): int { return f(v); }
println(apply(inc, 41));
```

## 5. 示例与验证

标准库示例目录：

- `examples/stdlib/hello.ziv`
- `examples/stdlib/io_demo.ziv`
- `examples/stdlib/math_demo.ziv`
- `examples/stdlib/string_demo.ziv`
- `examples/stdlib/array_demo.ziv`
- `examples/stdlib/container_demo.ziv`
- `examples/stdlib/js_demo.ziv`
- `examples/stdlib/filesystem_demo.ziv`
- `examples/stdlib/net_demo.ziv`
- `examples/stdlib/crypto_demo.ziv`
- `examples/stdlib/encoding_demo.ziv`

运行全部测试：

```bash
cargo test --workspace --all-targets
```

单独验证示例：

```bash
./target/debug/ziv examples/stdlib/hello.ziv -o /tmp/hello && /tmp/hello
```

## 6. 当前实现边界

- 已稳定可执行：`print/println` 输出链路、`container` 运行时行为
- 已完成注册与编译链路覆盖：其余标准库函数（可用于语义检查、IR 构建、接口对齐）
- 面向生产的文件系统/网络/加解密真实执行能力，建议通过宿主运行时对象或外部链接库补齐

## 7. 推荐实践

- 把标准库函数当作稳定 API 面设计代码
- 对涉及外部副作用（文件、网络、加密）的函数，在当前阶段优先通过测试桩/宿主注入来验证
- 在 examples 中保留最小可运行样例，和 CI 测试保持一致
