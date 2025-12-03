@echo off
echo.
echo ========================================
echo Running Tests
echo ========================================
echo.

echo Running unit tests...
cargo test --release --verbose
if %errorlevel% neq 0 (
    echo [ERROR] Tests failed
    exit /b 1
)

echo.
echo ========================================
echo [SUCCESS] All tests passed!
echo ========================================

