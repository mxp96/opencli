@echo off
echo.
echo ========================================
echo Checking Code Quality
echo ========================================
echo.

echo [1/3] Checking code formatting...
cargo fmt --all -- --check
if %errorlevel% neq 0 (
    echo [ERROR] Code is not formatted correctly
    echo.
    echo Run this to fix:
    echo   cargo fmt --all
    echo   or: scripts\format.bat
    exit /b 1
)
echo [OK] Code formatting is correct
echo.

echo [2/3] Running Clippy linter...
cargo clippy --all-targets --all-features -- -D warnings
if %errorlevel% neq 0 (
    echo [ERROR] Clippy found issues
    exit /b 1
)
echo [OK] No Clippy warnings
echo.

echo [3/3] Checking compilation...
cargo check --all-targets --all-features
if %errorlevel% neq 0 (
    echo [ERROR] Compilation check failed
    exit /b 1
)
echo [OK] Code compiles successfully
echo.

echo ========================================
echo [SUCCESS] All checks passed!
echo ========================================
