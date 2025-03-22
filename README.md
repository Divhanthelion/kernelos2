# KernelOS - Browser-based Desktop Environment

A WebAssembly-based desktop environment that runs in the browser.

## Features

- **File Management**: Create, modify, and delete files and directories with persistent storage
- **Terminal**: Execute basic commands like ls, cd, pwd, cat, mkdir, rm, etc.
- **Text Editor**: Create and edit text files with auto-save capability
- **Clock**: Display current time and date
- **Image Viewer**: View images (currently with placeholder functionality)
- **Window Management**: Move, minimize, and focus windows

## Getting Started

### Prerequisites

You'll need to have the following installed:

- Rust and Cargo (https://rustup.rs/)
- wasm-pack (https://rustwasm.github.io/wasm-pack/installer/)
- A web server or development server (like trunk)

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/kernelos.git
   cd kernelos
   ```

2. Install trunk for development server:
   ```
   cargo install trunk
   ```

3. Run the application:
   ```
   trunk serve
   ```

4. Open your browser and navigate to `http://localhost:8080`

## Building for Production

To build the project for production:

```
trunk build --release
```

The output will be in the `dist` directory.

## Project Structure

- `src/components/` - UI components (desktop, windows, applications)
- `src/filesystem.rs` - File system implementation with local storage backend
- `index.html` - Main HTML template

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Yew framework for Rust/Wasm
- Rust community for documentation and libraries 