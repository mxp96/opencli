@echo off
echo Setting up test scenarios...

mkdir test-scenarios\install 2>nul
mkdir test-scenarios\remove 2>nul
mkdir test-scenarios\build 2>nul
mkdir test-scenarios\legacy 2>nul
mkdir test-scenarios\versions 2>nul
mkdir test-scenarios\integration 2>nul

for %%d in (install remove build legacy versions integration) do (
    echo main() { print("Test scenario"); } > test-scenarios\%%d\gamemode.pwn
)

echo Test scenarios created successfully

