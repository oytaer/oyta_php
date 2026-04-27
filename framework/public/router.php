<?php

/**
 * OYTAPHP 快速测试文件
 * 
 * 本文件用于快速测试路由和控制器
 * 在传统PHP环境下会显示提示信息
 * 
 * 使用方法：
 * 1. 进入项目目录
 * 2. 运行命令：./oyta run
 * 3. 访问：http://localhost:8000
 */

if (php_sapi_name() === 'cli') {
    echo "===========================================\n";
    echo "  OYTAPHP - Rust驱动的PHP框架\n";
    echo "===========================================\n\n";
    echo "请使用以下命令启动服务：\n";
    echo "  ./oyta run           # 默认端口8000\n";
    echo "  ./oyta run -p 8080   # 指定端口\n\n";
    echo "更多信息请访问：https://oyta.dev/docs\n";
    exit(1);
}

echo "OYTAPHP is running!\n";
echo "Please use './oyta run' command to start the server.\n";
