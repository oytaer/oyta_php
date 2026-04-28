# 反射模块 (Reflection)

## 模块概述

反射模块提供完整的 PHP Reflection 功能，包括 ReflectionClass、ReflectionMethod、ReflectionProperty 等。

## 模块结构

```
reflection/
├── mod.rs          # 模块入口
├── class.rs        # ReflectionClass
├── method.rs       # ReflectionMethod
├── property.rs     # ReflectionProperty
├── parameter.rs    # ReflectionParameter
├── function.rs     # ReflectionFunction
├── attribute.rs    # ReflectionAttribute
├── type.rs         # 类型反射
├── enum_ref.rs     # 枚举反射
└── metadata.rs     # 元数据管理
```

## 主要类型

| 类型 | 说明 |
|------|------|
| ReflectionClass | 类反射 |
| ReflectionMethod | 方法反射 |
| ReflectionProperty | 属性反射 |
| ReflectionParameter | 参数反射 |
| ReflectionFunction | 函数反射 |
| ReflectionAttribute | 注解反射 |
| ReflectionNamedType | 命名类型反射 |
| ReflectionUnionType | 联合类型反射 |
| ReflectionEnum | 枚举反射 |

## 使用示例

### 类反射

```php
<?php
$class = new ReflectionClass('UserController');

// 获取类名
echo $class->getName();  // UserController

// 获取方法
$methods = $class->getMethods();

// 获取属性
$properties = $class->getProperties();

// 创建实例
$instance = $class->newInstance();
```

### 方法反射

```php
<?php
$method = new ReflectionMethod('UserController', 'index');

// 获取方法名
echo $method->getName();  // index

// 获取参数
$parameters = $method->getParameters();

// 调用方法
$method->invoke($instance);
```

### 属性反射

```php
<?php
$property = new ReflectionProperty('User', 'name');

// 获取属性名
echo $property->getName();  // name

// 获取属性值
$value = $property->getValue($instance);

// 设置属性值
$property->setValue($instance, '新值');
```

### 注解反射

```php
<?php
$class = new ReflectionClass('UserController');
$attributes = $class->getAttributes();

foreach ($attributes as $attr) {
    echo $attr->getName();
    print_r($attr->getArguments());
}
```

## 技术实现

- 完整的反射 API
- 支持注解反射
- 支持类型反射
- 支持枚举反射
