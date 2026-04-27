@echo off
REM OYTAPHP Windows 启动脚本
REM 自动运行 Windows 版本的二进制文件
REM
REM 使用方法:
REM   oyta run          REM 启动开发服务器
REM   oyta --help       REM 查看帮助
REM

set SCRIPT_DIR=%~dp0
set BINARY=%SCRIPT_DIR%vendor\oyta_php\bin\windows-x86_64\oyta.exe

if not exist "%BINARY%" (
    echo 错误：找不到 Windows 二进制文件：%BINARY%
    exit /b 1
)

"%BINARY%" %*
