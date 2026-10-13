# RUSTZILLA

A lightweight browser engine written in Rust that renders HTML and CSS into a native GUI application.

## Overview

Rustzilla is an educational browser engine implementation that demonstrates the core components of web rendering: HTML parsing, CSS styling, layout calculation, and visual rendering. Built with Rust and the egui framework, it provides a working example of how browsers transform markup into pixels on screen.

## Features

- **HTML Parsing**: Parse HTML documents into a DOM tree structure
- **CSS Support**:
    - External stylesheets via `<link>` tags
    - Inline `<style>` blocks
    - Inline `style` attributes
    - CSS selector matching (tag, class, ID, universal)
    - CSS specificity and cascade rules
    - Property inheritance
- **Layout Engine**:
    - Block and inline layout modes
    - Flexbox support
    - Box model (margin, padding)
    - Text wrapping and multi-line support
    - Percentage and pixel-based units
- **Rendering**:
    - Background colors
    - Text rendering with custom fonts
    - Text alignment (left, center, right)
    - Font weights (regular, bold)
- **Interactive Features**:
    - Clickable links with navigation
    - Scrollable viewport
    - Dynamic window resizing

## Usage

### Running the Browser

```bash
cargo run --release
```

The browser will launch and load the default page from `examples/showcase_page/index.html`.

### Project Structure

```
rustzilla/
├── src/
│   ├── main.rs           # Entry point
│   ├── dom/              # DOM implementation
│   ├── style/            # CSS engine
│   ├── layout/           # Layout calculations
│   └── gui/              # Rendering & UI
├── examples/
│   └── showcase_page/    # Sample HTML pages
│       ├── index.html
│       ├── styles.css
│       ├── times.ttf
│       └── times-bold.ttf
└── Cargo.toml
```

### Creating HTML Pages

Place your HTML files in a directory with the required font files:

```
my_page/
├── index.html
├── styles.css (optional)
├── times.ttf
└── times-bold.ttf
```

Update `app.rs` to point to your page directory, or click links within rendered pages to navigate.

## Supported HTML Tags

- Layout: `<div>`, `<body>`, `<html>`
- Text: `<p>`, `<span>`, `<h1>`, `<h2>`, `<h3>`
- Formatting: `<b>`, `<strong>`
- Interactive: `<a href="...">`

## Supported CSS Properties

- **Display**: `block`, `inline`, `flex`, `none`
- **Box Model**: `width`, `height`, `margin-*`, `padding-*`, `border-width`
- **Colors**: `color`, `background-color`
- **Typography**: `font-size`, `font-weight`, `text-align`
- **Flexbox**: `flex-direction`


## How It Works

1. **Parse HTML**: The HTML parser converts markup into a tree of `ParsedElement` nodes
2. **Extract CSS**: CSS is collected from `<link>` tags, `<style>` blocks, and inline attributes
3. **Build DOM**: Elements are converted to `DomNode` objects with computed styles
4. **Calculate Layout**: The Taffy layout engine computes positions and sizes
5. **Render**: The painter draws each element to a pixel buffer
6. **Display**: The buffer is shown in an egui window with scroll support


## License

This project is provided as-is for educational purposes.

