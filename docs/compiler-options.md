# Pawn Compiler Options

> **Note:** This documentation is based on the official [pawn-lang/compiler wiki](https://github.com/pawn-lang/compiler/wiki/Options). For the most up-to-date information, please refer to the original source.

## Overview

The Pawn compiler (`pawncc`) accepts various command-line options to control compilation behavior, debugging, optimization, and output.

## Command Line Syntax

```bash
pawncc [options] <source_file.pwn>
```

Options can be specified with either `-` or `/` prefix on Windows.

---

## Options Reference

### `-a` - Assembly Output

Output assembly listing to `<script_name>.asm` during compilation.

```bash
pawncc -a script.pwn
```

Useful for inspecting generated P-code/bytecode.

---

### `-d<N>` - Debug Level

Controls debug information and runtime checks.

| Level | Description |
|-------|-------------|
| `-d0` | No debug symbols, no runtime checks (smallest/fastest output) |
| `-d1` | Runtime checks (bounds checking) without debug symbols |
| `-d2` | Full debug information and all runtime checks |
| `-d3` | Same as `-d2` but disables code optimization |

**Example:**
```bash
pawncc -d3 script.pwn   # Full debug, no optimization
pawncc -d0 script.pwn   # Production build, no debug
```

**Recommended:**
- Development: `-d3` (easier debugging)
- Production: `-d0` or `-d1` (better performance)

---

### `-D<path>` - Output Directory

Sets the output directory for compiled files.

```bash
pawncc -D./build script.pwn
```

The compiled `.amx` file will be placed in the specified directory.

---

### `-e<file>` - Error File

Redirects error messages to a file instead of the console.

```bash
pawncc -eerrors.txt script.pwn
```

---

### `-i<path>` - Include Directory

Adds a directory to the include search path. Can be specified multiple times.

```bash
pawncc -i./include -i./pawno/include script.pwn
```

**Note:** Paths with spaces should be quoted.

---

### `-o<file>` - Output File

Specifies the output filename for the compiled script.

```bash
pawncc -o./gamemodes/mygame.amx script.pwn
```

---

### `-O<N>` - Optimization Level

Controls code optimization.

| Level | Description |
|-------|-------------|
| `-O0` | No optimization |
| `-O1` | Limited optimization (JIT-compatible only) |
| `-O2` | Full optimization (default) |

**Example:**
```bash
pawncc -O2 script.pwn   # Full optimization
pawncc -O0 script.pwn   # No optimization (for debugging)
```

---

### `-p<name>` - Prefix File

Specifies a "prefix" file to be included before the main source.

```bash
pawncc -pprefix.inc script.pwn
```

---

### `-r<name>` - Report File

Generates a report file with cross-reference information.

```bash
pawncc -rreport.txt script.pwn
```

---

### `-S<N>` - Stack/Heap Size

Sets the stack and heap size (in cells).

```bash
pawncc -S16384 script.pwn
```

Default is usually sufficient for most scripts.

---

### `-t<N>` - Tab Size

Sets the number of spaces per tab character for error reporting.

```bash
pawncc -t4 script.pwn
```

---

### `-v<N>` - Verbosity Level

Controls verbosity of compiler output.

| Level | Description |
|-------|-------------|
| `-v0` | Quiet (errors only) |
| `-v1` | Normal |
| `-v2` | Verbose (includes statistics) |

---

### `-w<N>` - Warning Control

Enables or disables specific warnings.

```bash
pawncc -w202 script.pwn    # Disable warning 202
```

Use with `+` or `-` suffix:
- `-w202+` - Enable warning 202
- `-w202-` - Disable warning 202

---

### `-Z[+/-]` - Run-time Compatibility

Toggles compatibility mode with the original Pawn/SA-MP compiler.

```bash
pawncc -Z+ script.pwn    # Enable compatibility mode
pawncc -Z- script.pwn    # Disable compatibility mode (default)
```

---

### `-;[+/-]` - Semicolon Requirement

Toggles requirement of semicolons at statement end.

```bash
pawncc -;+ script.pwn    # Require semicolons (recommended)
pawncc -;- script.pwn    # Semicolons optional
```

---

### `-(+/-)` - Parentheses Requirement

Toggles requirement of parentheses in some expression contexts.

```bash
pawncc -(+ script.pwn    # Require parentheses
```

---

### `-\[+/-]` - Backslash Escape

Enables/disables interpretation of backslash as escape character in strings.

```bash
pawncc -\+ script.pwn    # Enable backslash escapes
```

---

### `symbol=value` - Define Constants

Defines a constant symbol at compile time.

```bash
pawncc DEBUG=1 VERSION=100 script.pwn
```

Useful for conditional compilation:
```pawn
#if defined DEBUG
    print("Debug mode enabled");
#endif
```

---

## OpenCLI Default Arguments

OpenCLI uses these default compiler arguments in `opencli.toml`:

```toml
[build.args]
args = ["-d3", "-;+", "-(+", "-\\+", "-Z+"]
```

| Argument | Purpose |
|----------|---------|
| `-d3` | Full debug with no optimization |
| `-;+` | Require semicolons |
| `-(+` | Require parentheses |
| `-\+` | Enable backslash escapes |
| `-Z+` | Enable compatibility mode |

### Customizing Arguments

Edit your `opencli.toml` to customize compiler arguments:

```toml
[build.args]
# Production build with optimization
args = ["-d0", "-O2", "-;+", "-(+", "-\\+"]
```

---

## Common Use Cases

### With Custom Defines
```bash
opencli build  # Uses opencli.toml settings
```

Or manually:
```bash
pawncc -d3 -;+ -(+ -\+ -Z+ -i./include script.pwn
```

---

## See Also

- [Official Pawn Compiler Wiki](https://github.com/pawn-lang/compiler/wiki/Options)
- [open.mp Documentation](https://open.mp/docs)
- [OpenCLI Configuration](../README.md#configuration)