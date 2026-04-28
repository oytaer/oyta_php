# FastCGI 模块 (FastCGI)

## 模块概述

FastCGI 模块实现 FastCGI 协议，让 OYTAPHP 可以像 PHP-FPM 一样被 Nginx 调用，用户可以像配置 PHP 网站一样简单部署。

## 模块结构

```
fastcgi/
├── mod.rs          # 模块入口
├── protocol.rs     # FastCGI 协议
├── server.rs       # FastCGI 服务器
└── handler.rs      # 请求处理器
```

## 主要类型

| 类型 | 说明 |
|------|------|
| FastCGIServer | FastCGI 服务器 |
| FastCGIHandler | 请求处理器 |

## 使用示例

### 启动 FastCGI 服务器

```bash
oyta fastcgi --socket /var/run/oyta.sock
```

### Nginx 配置

```nginx
server {
    listen 80;
    server_name example.com;
    root /var/www/public;
    
    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }
    
    location ~ \.php$ {
        fastcgi_pass unix:/var/run/oyta.sock;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }
}
```

## 技术实现

- FastCGI 协议实现
- Unix Socket 支持
- TCP Socket 支持
- 与 Nginx 兼容
