# LightScript 语言规范 v0.1

## 目录

1. [概述](#概述)
2. [词法结构](#词法结构)
3. [类型系统](#类型系统)
4. [表达式](#表达式)
5. [语句](#语句)
6. [函数](#函数)
7. [模块系统](#模块系统)
8. [内存管理](#内存管理)

---

## 概述

LightScript 是一门静态类型、编译型编程语言，设计目标是：

- **简洁性**：去除 JavaScript 的历史包袱
- **安全性**：强类型 + 所有权系统
- **性能**：编译到原生代码
- **表达力**：现代语法特性

### 设计原则

1. **显式优于隐式**：拒绝类型强制转换
2. **安全优于方便**：默认不可变
3. **清晰优于简洁**：可读性第一
4. **性能优于灵活**：编译时优化

---

## 词法结构

### 关键字

```lightscript
let const function return
if else match
while for break continue
true false null
import export
async await
```

### 标识符

```text
identifier = [a-zA-Z_][a-zA-Z0-9_]*
```

示例：
```lightscript
foo
_bar
CamelCase
snake_case
myVar123
```

### 字面量

#### 数字
```lightscript
42              // 整数
3.14            // 浮点数
0xFF            // 十六进制
0b1010          // 二进制
0o755           // 八进制
1_000_000       // 数字分隔符
```

#### 字符串
```lightscript
"hello"                     // 普通字符串
'world'                     // 单引号字符串
`hello ${name}`             // 模板字符串
```

#### 布尔值
```lightscript
true
false
```

### 注释

```lightscript
// 单行注释

/*
  多行注释
*/

/// 文档注释
```

---

## 类型系统

### 基本类型

| 类型 | 描述 | 示例 |
|------|------|------|
| `number` | 64位浮点数 | `42`, `3.14` |
| `int` | 64位整数 | `42` |
| `string` | UTF-8字符串 | `"hello"` |
| `boolean` | 布尔值 | `true`, `false` |
| `null` | 空值 | `null` |

### 复合类型

#### 数组
```lightscript
const numbers: number[] = [1, 2, 3];
const matrix: number[][] = [[1, 2], [3, 4]];
```

#### 对象
```lightscript
type Person = {
    name: string,
    age: number,
};

const person: Person = {
    name: "Alice",
    age: 30,
};
```

#### 元组
```lightscript
const point: [number, number] = [10, 20];
```

#### 函数类型
```lightscript
type BinaryOp = (a: number, b: number) => number;
```

### 类型推断

LightScript 支持类型推断：

```lightscript
const x = 10;              // 推断为 int
const y = 3.14;            // 推断为 number
const add = (a, b) => a + b;  // 推断参数和返回类型
```

---

## 表达式

### 算术运算

```lightscript
1 + 2       // 加法
3 - 1       // 减法
2 * 3       // 乘法
10 / 2      // 除法
7 % 3       // 取模
2 ** 3      // 幂运算
```

### 比较运算

```lightscript
1 == 1      // 相等（值比较）
1 != 2      // 不等
1 < 2       // 小于
1 <= 2      // 小于等于
1 > 0       // 大于
1 >= 0      // 大于等于
```

### 逻辑运算

```lightscript
true && false   // 与
true || false   // 或
!true           // 非
```

### 解构

```lightscript
// 数组解构
const [a, b, c] = [1, 2, 3];

// 对象解构
const { name, age } = person;

// 嵌套解构
const { address: { city } } = person;
```

### 展开运算符

```lightscript
const arr1 = [1, 2, 3];
const arr2 = [...arr1, 4, 5];  // [1, 2, 3, 4, 5]

const obj1 = { a: 1 };
const obj2 = { ...obj1, b: 2 };  // { a: 1, b: 2 }
```

---

## 语句

### 变量声明

```lightscript
// 不可变变量（推荐）
const x = 10;
const y: number = 20;

// 可变变量
let counter = 0;
counter = 1;
```

### 条件语句

```lightscript
// if-else
if (condition) {
    // ...
} else if (another) {
    // ...
} else {
    // ...
}

// 三元运算符
const result = condition ? a : b;
```

### 循环语句

```lightscript
// while
while (condition) {
    // ...
}

// for
for (let i = 0; i < 10; i++) {
    console.log(i);
}

// for-of
for (const item of array) {
    console.log(item);
}

// for-in
for (const key in object) {
    console.log(key);
}
```

### 模式匹配

```lightscript
match (value) {
    0 => "zero",
    1 | 2 | 3 => "small",
    n if n < 10 => "medium",
    _ => "large"
}
```

---

## 函数

### 函数声明

```lightscript
// 普通函数
function add(a: number, b: number): number {
    return a + b;
}

// 箭头函数
const multiply = (a: number, b: number): number => a * b;

// 多行箭头函数
const factorial = (n: number): number => {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
};
```

### 默认参数

```lightscript
function greet(name: string, greeting: string = "Hello") {
    return `${greeting}, ${name}!`;
}
```

### 剩余参数

```lightscript
function sum(...numbers: number[]): number {
    return numbers.reduce((a, b) => a + b, 0);
}
```

### 闭包

```lightscript
function counter() {
    let count = 0;
    return () => {
        count++;
        return count;
    };
}

const c = counter();
c();  // 1
c();  // 2
```

---

## 模块系统

### 导出

```lightscript
// math.ls
export const PI = 3.14159;

export function add(a, b) {
    return a + b;
}

export default class Calculator {
    // ...
}
```

### 导入

```lightscript
// main.ls
import { PI, add } from "./math.ls";
import Calculator from "./calculator.ls";
import * as Math from "./math.ls";
```

---

## 内存管理

### 所有权规则

1. 每个值都有一个所有者
2. 同一时刻只能有一个所有者
3. 所有者离开作用域时，值被释放

```lightscript
let s1 = "hello";
let s2 = s1;  // s1 不再有效

function take(s: string) {
    // s 在这里被释放
}

take(s2);  // s2 不再有效
```

### 借用

```lightscript
function len(s: &string): number {
    return s.length;  // 只读借用
}

let s = "hello";
let length = len(&s);  // s 仍然有效
```

---

## 标准库

### console

```lightscript
console.log("Hello");
console.error("Error");
console.warn("Warning");
```

### Array

```lightscript
const arr = [1, 2, 3];
arr.push(4);
arr.map(x => x * 2);
arr.filter(x => x > 1);
arr.reduce((a, b) => a + b, 0);
```

### String

```lightscript
const s = "hello";
s.toUpperCase();
s.split("");
s.trim();
```

---

## 未来计划

- [ ] 泛型支持
- [ ] 异步迭代器
- [ ] 宏系统
- [ ] 模式匹配增强
- [ ] 类型系统增强

---

*LightScript v0.1 - 2026*
