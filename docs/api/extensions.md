# 扩展模块文档

## GD 图像处理模块

```php
<?php
// 创建图像
$image = new GdImage();
$image->new(800, 600);

// 加载图像
$image->load('input.jpg');

// 调整大小
$image->resize(400, 300);

// 裁剪
$image->crop(0, 0, 200, 200);

// 旋转
$image->rotate(90);

// 滤镜
$image->filter('grayscale');
$image->filter('blur', 5);

// 添加文字
$image->text('Hello', 10, 10, ['color' => 'red']);

// 绘制图形
$image->line(0, 0, 100, 100, 'blue');
$image->rectangle(10, 10, 100, 100, 'green');
$image->ellipse(50, 50, 80, 60, 'red');

// 保存
$image->save('output.jpg', 80); // 80% 质量

// 输出到浏览器
$image->output('png');

// 转换为 Base64
$base64 = $image->toBase64('png');
```

## XML 处理模块

```php
<?php
// DOM 方式
$doc = new DOMDocument();
$doc->load('file.xml');
$doc->loadXML($xmlString);

// 创建元素
$root = $doc->createElement('root');
$doc->appendChild($root);

// XPath 查询
$xpath = new DOMXPath($doc);
$nodes = $xpath->query('//item[@id="1"]');

// SimpleXML 方式
$xml = simplexml_load_file('file.xml');
$xml = simplexml_load_string($xmlString);

// 访问元素
echo $xml->item[0]->title;

// XMLReader 流式解析
$reader = new XMLReader();
$reader->open('large.xml');
while ($reader->read()) {
    if ($reader->nodeType == XMLReader::ELEMENT) {
        // 处理元素
    }
}

// XMLWriter 生成 XML
$writer = new XMLWriter();
$writer->openMemory();
$writer->startDocument();
$writer->startElement('root');
$writer->writeElement('item', 'value');
$writer->endElement();
echo $writer->outputMemory();
```

## Mbstring 多字节字符串模块

```php
<?php
// 字符串长度
$len = mb_strlen('你好世界', 'UTF-8');

// 截取
$sub = mb_substr('你好世界', 0, 2, 'UTF-8');

// 查找位置
$pos = mb_strpos('你好世界', '世', 0, 'UTF-8');

// 大小写转换
$upper = mb_strtoupper('hello', 'UTF-8');
$lower = mb_strtolower('HELLO', 'UTF-8');

// 编码转换
$utf8 = mb_convert_encoding($str, 'UTF-8', 'GBK');

// 字符统计
$count = mb_count_chars($str, 'UTF-8');
```

## DateTime 日期时间模块

```php
<?php
// 创建日期
$date = new DateTime();
$date = new DateTime('2024-01-01');
$date = DateTime::createFromFormat('Y-m-d', '2024-01-01');

// 格式化
echo $date->format('Y-m-d H:i:s');

// 修改
$date->modify('+1 day');
$date->modify('+1 month');
$date->modify('next Monday');

// 比较
$diff = $date1->diff($date2);
echo $diff->days;

// 时区
$date->setTimezone(new DateTimeZone('Asia/Shanghai'));

// DateInterval
$interval = new DateInterval('P1D'); // 1天
$interval = DateInterval::createFromDateString('1 day');

// DatePeriod 迭代
$period = new DatePeriod($start, $interval, $end);
foreach ($period as $date) {
    echo $date->format('Y-m-d');
}
```

## Phar 归档模块

```php
<?php
// 创建 Phar
$phar = new Phar('app.phar');
$phar->buildFromDirectory('src/');
$phar->setStub('<?php __HALT_COMPILER();');

// 解压 Phar
$phar->extractTo('extracted/');

// 添加文件
$phar->addFile('config.php');
$phar->addFromString('readme.txt', '内容');

// 签名
$phar->setSignatureAlgorithm(Phar::SHA256);

// 读取 Phar 内容
$content = file_get_contents('phar://app.phar/config.php');
```

## Serialize 序列化模块

```php
<?php
// MessagePack
$packed = Serialize::encode($data, 'msgpack');
$data = Serialize::decode($packed, 'msgpack');

// Bincode
$encoded = Serialize::encodeBincode($data);
$decoded = Serialize::decodeBincode($encoded);

// CBOR
$encoded = Serialize::encodeCbor($data);
$decoded = Serialize::decodeCbor($encoded);

// Postcard
$encoded = Serialize::encodePostcard($data);
$decoded = Serialize::decodePostcard($encoded);

// 获取序列化后大小
$size = Serialize::getSize($data, 'msgpack');
```
