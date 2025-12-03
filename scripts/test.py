#!/usr/bin/env python3
import subprocess
import sys

def main():
    print()
    print("========================================")
    print("Running Tests")
    print("========================================")
    print()

    # Run unit tests using Cargo
    print("Running unit tests...")
    try:
        # Execute Cargo test command in release mode with verbose output
        subprocess.run("cargo test --release --verbose", shell=True, check=True)
    except subprocess.CalledProcessError:
        # If any test fails, display error message and exit
        print()
        print("========================================")
        print("[ERROR] Tests failed!")
        print("========================================")
        sys.exit(1)

    # If all tests pass successfully
    print()
    print("========================================")
    print("[SUCCESS] All tests passed!")
    print("========================================")

if __name__ == "__main__":
    try:
        # Execute the main function
        main()
    except KeyboardInterrupt:
        # Handle user interruption (Ctrl + C)
        print("\n[ABORTED] User interrupted")
        sys.exit(1)
