# LightLang 语言规范 v0.1

## 目录

1. [概述](#概述)
2. [词法结构](#词法结构)
3. [类型系统](#类型系统)
4. [所有权系统](#所有权系统)
5. [内存布局](#内存布局)
6. [表达式](#表达式)
7. [语句](#语句)
8. [函数](#函数)
9. [模块系统](#模块系统)
10. [系统编程](#系统编程)

---

## 概述

LightLang 是一门**系统级编程语言**，设计目标：

- **内存安全**：通过所有权系统在编译时保证内存安全
- **零成本抽象**：高级特性不影响运行时性能
- **原生编译**：直接编译为 ELF 格式的可执行文件
- **系统访问**：支持内联汇编、系统调用、指针操作
- **C 兼容**：可以与 C 代码无缝互操作

### 设计原则

1. **安全默认**：默认安全的操作，unsafe 显式标记
2. **显式优于隐式**：拒绝魔法行为
3. **性能优于方便**：零成本抽象
4. **清晰优于简洁**：可读性第一

---

## 词法结构

### 关键字

```lightlang
// 声明
let const mut static
fn struct enum trait impl
type alias

// 控制流
if else match
while for loop break continue return

// 类型
true false null
unsafe

// 模块
mod use pub

// 其他
as in where ref
```

### 标识符

```text
identifier = [a-zA-Z_][a-zA-Z0-9_]*
```

示例：
```lightlang
foo
_bar
CamelCase
snake_case
myVar123
```

### 字面量

#### 整数
```lightlang
42              // 默认 i32
127i8           // i8
255u8           // u8
1024i64         // i64
0xFF            // 十六进制
0b1010          // 二进制
0o755           // 八进制
1_000_000       // 数字分隔符
```

#### 浮点数
```lightlang
3.14            // 默认 f64
2.5f32          // f32
1.0e10          // 科学计数法
```

#### 字符和字符串
```lightlang
'a'                         // 字符
"hello"                     // 字符串
`hello ${name}`             // 模板字符串（编译时）
```

#### 布尔值
```lightlang
true
false
```

### 注释

```lightlang
// 单行注释

/*
  多行注释
*/

/// 文档注释
```

---

## 类型系统

### 基本类型

| 类型 | 大小 | 描述 | 示例 |
|------|------|------|------|
| `i8` | 1字节 | 有符号8位整数 | `-128i8` |
| `i16` | 2字节 | 有符号16位整数 | `-32768i16` |
| `i32` | 4字节 | 有符号32位整数 | `-42i32` |
| `i64` | 8字节 | 有符号64位整数 | `-9999i64` |
| `i128` | 16字节 | 有符号128位整数 | `-1i128` |
| `u8` | 1字节 | 无符号8位整数 | `255u8` |
| `u16` | 2字节 | 无符号16位整数 | `65535u16` |
| `u32` | 4字节 | 无符号32位整数 | `42u32` |
| `u64` | 8字节 | 无符号64位整数 | `9999u64` |
| `u128` | 16字节 | 无符号128位整数 | `1u128` |
| `f32` | 4字节 | 32位浮点数 | `3.14f32` |
| `f64` | 8字节 | 64位浮点数 | `3.14159265359f64` |
| `bool` | 1字节 | 布尔值 | `true`, `false` |
| `char` | 4字节 | Unicode字符 | `'字'` |

### 复合类型

#### 数组
```lightlang
// 固定大小数组
const numbers: [i32; 5] = [1, 2, 3, 4, 5];

// 数组切片
fn sum(arr: &[i32]) -> i32 {
    let mut total = 0;
    for i in arr {
        total += i;
    }
    return total;
}
```

#### 结构体
```lightlang
struct Point {
    x: f64,
    y: f64,
}

// 带方法
impl Point {
    fn new(x: f64, y: f64) -> Self {
        return Point { x, y };
    }
    
    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        return (dx * dx + dy * dy).sqrt();
    }
}
```

#### 枚举
```lightlang
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// 使用
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Result::Err("division by zero");
    }
    return Result::Ok(a / b);
}
```

#### 元组
```lightlang
const point: (f64, f64) = (10.0, 20.0);
const (x, y) = point;  // 解构
```

### 指针类型

```lightlang
// 引用（安全）
let x = 10;
let r = &x;      // 不可变引用
let r2 = &mut x; // 可变引用

// 原始指针（unsafe）
let raw: *const i32 = &x;
let mut_raw: *mut i32 = &mut x;
```

---

## 所有权系统

### 三大规则

1. **每个值都有一个所有者**
2. **同一时刻只能有一个所有者**
3. **所有者离开作用域时，值被释放**

### 所有权转移

```lightlang
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1 所有权转移给 s2
    
    // println(s1);  // ❌ 编译错误：s1 已失效
    println(s2);     // ✅ 正确
}
```

### 借用规则

```lightlang
// 1. 可以有多个不可变引用
let s = String::from("hello");
let r1 = &s;  // ✅
let r2 = &s;  // ✅

// 2. 或者一个可变引用
let mut s = String::from("hello");
let r = &mut s;  // ✅

// 3. 不能同时有可变和不可变引用
let mut s = String::from("hello");
let r1 = &s;      // ✅
let r2 = &mut s;  // ❌ 编译错误
```

### 生命周期

```lightlang
// 显式生命周期标注
fn longest<'a>(s1: &'a String, s2: &'a String) -> &'a String {
    if s1.len() > s2.len() {
        return s1;
    }
    return s2;
}
```

---

## 内存布局

### 内存布局控制

```lightlang
// C 兼容布局
#[repr(C)]
struct CStruct {
    x: i32,
    y: f64,
}

// 指定对齐
#[repr(align(16))]
struct Aligned {
    data: [u8; 32],
}

// 透明布局（与底层类型相同）
#[repr(transparent)]
struct Wrapper(u32);

// 无填充
#[repr(packed)]
struct Packed {
    a: u8,
    b: u32,  // 紧跟在 a 后面，无对齐填充
}
```

### 大小计算

```lightlang
use std::mem;

struct Point {
    x: f64,
    y: f64,
}

fn main() {
    println("Size of Point: {}", mem::size_of::<Point>());  // 16
    println("Align of Point: {}", mem::align_of::<Point>()); // 8
}
```

---

## 表达式

### 算术运算

```lightlang
1 + 2       // 加法
3 - 1       // 减法
2 * 3       // 乘法
10 / 2      // 除法
7 % 3       // 取模
2 ** 3      // 幂运算
```

### 位运算

```lightlang
0b1010 & 0b1100   // 与
0b1010 | 0b1100   // 或
0b1010 ^ 0b1100   // 异或
!0b1010           // 取反
0b1010 << 2       // 左移
0b1010 >> 2       // 右移
```

### 比较运算

```lightlang
1 == 1      // 相等
1 != 2      // 不等
1 < 2       // 小于
1 <= 2      // 小于等于
1 > 0       // 大于
1 >= 0      // 大于等于
```

### 逻辑运算

```lightlang
true && false   // 与
true || false   // 或
!true           // 非
```

---

## 语句

### 变量声明

```lightlang
// 不可变变量（默认）
let x = 10;
let y: i64 = 20;

// 可变变量
let mut counter = 0;
counter += 1;
```

### 条件语句

```lightlang
// if-else
if (condition) {
    // ...
} else if (another) {
    // ...
} else {
    // ...
}

// 表达式形式
let result = if (x > 0) {
    "positive"
} else {
    "non-positive"
};
```

### 循环语句

```lightlang
// while
while (condition) {
    // ...
}

// for
for i in 0..10 {
    println(i);
}

// loop
loop {
    if (done) {
        break;
    }
}

// 迭代器
for item in collection.iter() {
    println(item);
}
```

### 模式匹配

```lightlang
match (value) {
    0 => "zero",
    1 | 2 | 3 => "small",
    n if n < 10 => "medium",
    _ => "large"
}

// 解构匹配
match (option) {
    Option::Some(x) => println(x),
    Option::None => println("none"),
}
```

---

## 函数

### 函数声明

```lightlang
// 普通函数
fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

// 表达式函数体
fn square(x: i64) -> i64 {
    x * x
}

// 泛型函数
fn max<T: Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
```

### 默认参数

```lightlang
fn greet(name: &str, greeting: &str = "Hello") {
    println("{}, {}!", greeting, name);
}
```

### 闭包

```lightlang
let add = |a, b| a + b;
let square = |x| x * x;

// 多行闭包
let factorial = |n| {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
};
```

---

## 模块系统

### 模块定义

```lightlang
// math.ll
pub mod geometry {
    pub struct Point {
        pub x: f64,
        pub y: f64,
    }
    
    pub fn distance(p1: &Point, p2: &Point) -> f64 {
        let dx = p1.x - p2.x;
        let dy = p1.y - p2.y;
        return (dx * dx + dy * dy).sqrt();
    }
}
```

### 导入

```lightlang
use math::geometry::{Point, distance};

fn main() {
    let p1 = Point { x: 0.0, y: 0.0 };
    let p2 = Point { x: 3.0, y: 4.0 };
    println(distance(&p1, &p2));  // 5.0
}
```

---

## 系统编程

### 内联汇编

```lightlang
fn get_cycles() -> u64 {
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

fn halt() {
    unsafe {
        asm!("hlt");
    }
}
```

### 系统调用

```lightlang
// Linux x86_64 系统调用
fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 1,           // write syscall number
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") count,
            out("rax") ret,
        );
    }
    return ret;
}

fn sys_exit(status: i32) {
    unsafe {
        asm!(
            "syscall",
            in("rax") 60,          // exit syscall number
            in("rdi") status,
        );
    }
}
```

### 内存操作

```lightlang
use std::ptr;

fn main() {
    // 分配内存
    let mut buffer: *mut u8 = ptr::null_mut();
    unsafe {
        buffer = libc::malloc(1024);
        
        // 写入数据
        ptr::write(buffer, 42);
        
        // 读取数据
        let value = ptr::read(buffer);
        println(value);  // 42
        
        // 释放内存
        libc::free(buffer);
    }
}
```

### C FFI

```lightlang
// 声明 C 函数
extern "C" {
    fn printf(format: *const i8, ...) -> i32;
    fn malloc(size: usize) -> *mut void;
    fn free(ptr: *mut void);
}

// 使用
fn main() {
    unsafe {
        printf(b"Hello, C!\n".as_ptr());
    }
}
```

---

## 标准库

### std::mem

```lightlang
use std::mem;

fn main() {
    // 大小和对齐
    println(mem::size_of::<i64>());   // 8
    println(mem::align_of::<i64>());  // 8
    
    // 类型转换
    let bytes: [u8; 8] = mem::transmute(42i64);
    
    // 替换
    let mut x = 5;
    let old = mem::replace(&mut x, 10);
    println(old);  // 5
    println(x);    // 10
}
```

### std::ptr

```lightlang
use std::ptr;

fn main() {
    let mut x = 10;
    
    // 原始指针
    let ptr: *mut i32 = &mut x;
    
    unsafe {
        // 读写
        ptr::write(ptr, 20);
        let value = ptr::read(ptr);
        println(value);  // 20
        
        // 拷贝
        let mut y = 0;
        ptr::copy(&x, &mut y, 1);
    }
}
```

---

## 编译输出

### ELF 格式

LightLang 编译器直接生成 ELF 可执行文件：

```bash
$ llc hello.ll -o hello
$ file hello
hello: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked

$ readelf -h hello
ELF Header:
  Magic:   7f 45 4c 46 02 01 01 00
  Class:   ELF64
  Data:    2's complement, little endian
  Version: 1 (current)
  Type:    EXEC (Executable file)
  Machine: Advanced Micro Devices X86-64
```

### 可执行文件段

```bash
$ readelf -S hello
Section Headers:
  [Nr] Name    Type     Address          Off    Size
  [ 0]         NULL     0000000000000000 000000 000000
  [ 1] .text   PROGBITS 0000000000401000 001000 000100
  [ 2] .rodata PROGBITS 0000000000402000 002000 000050
  [ 3] .data   PROGBITS 0000000000403000 003000 000010
  [ 4] .bss    NOBITS   0000000000404000 004000 000010
```

---

## 未来计划

- [ ] 泛型约束增强
- [ ] 异步编程支持
- [ ] 宏系统
- [ ] 编译时计算
- [ ] 更多优化 Pass

---

*LightLang v0.1 - 2026*
