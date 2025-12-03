#!/usr/bin/env python3
import subprocess
import sys

def main():
    # Display an initial message
    print("Formatting Rust code...")

    try:
        # Run the Rust formatter (cargo fmt) across all packages and modules
        # Equivalent to: `cargo fmt --all`
        subprocess.run("cargo fmt --all", shell=True, check=True)

    except subprocess.CalledProcessError:
        # If cargo fmt fails (e.g., invalid Cargo project or formatting error)
        print("[ERROR] Failed to format code.")
        sys.exit(1)

    # If successful, display success message and usage hint
    print("[SUCCESS] Code formatted successfully!")
    print()
    print("Run this to check formatting:")
    print("  cargo fmt --all -- --check")

if __name__ == "__main__":
    try:
        # Execute the main function
        main()
    except KeyboardInterrupt:
        # Handle user interruption gracefully (Ctrl+C)
        print("\n[ABORTED] User interrupted")
        sys.exit(1)
