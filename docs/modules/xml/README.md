# XML 处理模块 (XML)

## 模块概述

XML 处理模块提供完整的 PHP XML 处理功能，包括 DOM、SimpleXML、XMLReader、XMLWriter 等。

## 模块结构

```
xml/
├── mod.rs          # 模块入口
├── dom.rs          # DOMDocument
├── simplexml.rs    # SimpleXML
├── reader.rs       # XMLReader
├── writer.rs       # XMLWriter
├── parser.rs       # XML 解析器
└── xpath.rs        # XPath 查询
```

## 主要类型

| 类型 | 说明 |
|------|------|
| XmlDocument | DOM 文档 |
| SimpleXmlElement | SimpleXML 元素 |
| XmlReader | XML 读取器 |
| XmlWriter | XML 写入器 |
| XPath | XPath 查询 |

## 使用示例

### DOM 操作

```php
<?php
// 创建 XML 文档
$doc = new DOMDocument('1.0', 'UTF-8');
$doc->formatOutput = true;

// 创建元素
$root = $doc->createElement('root');
$doc->appendChild($root);

$user = $doc->createElement('user');
$user->setAttribute('id', '1');
$user->appendChild($doc->createElement('name', '张三'));
$root->appendChild($user);

// 输出 XML
echo $doc->saveXML();
```

### SimpleXML

```php
<?php
// 加载 XML
$xml = simplexml_load_string($xmlString);

// 访问元素
echo $xml->user->name;

// 遍历元素
foreach ($xml->user as $user) {
    echo $user->name;
}
```

### XPath 查询

```php
<?php
$doc = new DOMDocument();
$doc->loadXML($xmlString);

$xpath = new DOMXPath($doc);
$nodes = $xpath->query('//user[@id="1"]/name');

foreach ($nodes as $node) {
    echo $node->nodeValue;
}
```

## 技术实现

- 基于 roxmltree 库
- 支持 DOM 操作
- 支持 SimpleXML
- 支持 XPath 查询
