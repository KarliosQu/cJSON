@echo off
REM Rust 项目编译脚本
REM 执行 cargo build 命令进行编译
REM 编译成功时返回退出码 0，编译失败时返回非 0 退出码并输出错误信息

cd /d "D:\Work\调研内容\Rust AI 实践\2026-1-23 灵犀自生成json实践\code\LX-json"
cargo build