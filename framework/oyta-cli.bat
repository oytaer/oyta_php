@echo off
REM OYTAPHP CLI Windows 启动脚本
REM 自动运行 Windows 版本的 CLI 二进制文件
REM
REM 使用方法:
REM   oyta-cli new myapp    REM 创建新项目
REM   oyta-cli --help       REM 查看帮助
REM

set SCRIPT_DIR=%~dp0
set BINARY=%SCRIPT_DIR%vendor\oyta_php\bin\windows-x86_64\oyta-cli.exe

if not exist "%BINARY%" (
    echo 错误：找不到 Windows CLI 二进制文件：%BINARY%
    exit /b 1
)

"%BINARY%" %*
