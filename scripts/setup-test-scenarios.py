#!/usr/bin/env python3
import os
import sys
from pathlib import Path

def main():
    print("Setting up test scenarios...")

    base_dir = Path("test-scenarios")
    scenario_names = [
        "install",
        "remove",
        "build",
        "legacy",
        "versions",
        "integration",
    ]

    # Create scenario directories
    for name in scenario_names:
        dir_path = base_dir / name
        dir_path.mkdir(parents=True, exist_ok=True)

    # Create a sample gamemode.pwn file in each directory
    for dir_path in base_dir.iterdir():
        if dir_path.is_dir():
            file_path = dir_path / "gamemode.pwn"
            with open(file_path, "w", encoding="utf-8") as f:
                f.write('main() { print("Test scenario"); }')

    print("Test scenarios created successfully")

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n[ABORTED] User interrupted")
        sys.exit(1)
    except Exception as e:
        print(f"[ERROR] {e}")
        sys.exit(1)
