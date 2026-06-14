@echo off
setlocal
cd /d "%~dp0"
set "PATH=C:\msys64\mingw32\bin;%PATH%"
cargo build --target i686-pc-windows-gnu
if errorlevel 1 exit /b %errorlevel%
copy /Y "target\i686-pc-windows-gnu\debug\wc3launcher.exe" "D:\AgentCoding\WC3Extender\deploy\wc3extender\wc3launcher.exe"
