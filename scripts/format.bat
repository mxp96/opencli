@echo off
echo Formatting Rust code...
cargo fmt --all
if %errorlevel% neq 0 (
    echo [ERROR] Failed to format code
    exit /b 1
)
echo [SUCCESS] Code formatted successfully!
echo.
echo Run this to check formatting:
echo   cargo fmt --all -- --check
