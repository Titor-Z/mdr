# mdr — Markdown Renderer

> A modern, self-contained markdown renderer for the terminal.

## Features

- **Built-in pager** — no need for `less` or `more`
- Syntax highlighting for code blocks
- List support (ordered and unordered)
- Blockquotes, horizontal rules, and more
- HTTP server mode *(coming soon)*

## Code Example

Here's a Rust function:

```rust
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    let msg = greet("world");
    println!("{}", msg);
}
```

And some **Python** too:

```python
def fibonacci(n):
    a, b = 0, 1
    for _ in range(n):
        yield a
        a, b = b, a + b

for i, num in enumerate(fibonacci(10)):
    print(f"fib({i}) = {num}")
```

## Lists

### Shopping list (unordered)

- Apples
- Bananas
- Cherries
  - Dark cherries
  - Rainier cherries
- Dates

### Top 3 programming languages (ordered)

1. Rust
2. Go
3. Python

## Inline styling

This is **bold**, this is *italic*, and this is ~~strikethrough~~.

Here's some `inline code` for you.

A link: [mdr on GitHub](https://github.com/titor/mdr)

## Table Example

| Name    | Age | City     |
|---------|-----|----------|
| Alice   | 30  | Tokyo    |
| Bob     | 25  | New York |
| Charlie | 35  | London   |

---

> "The journey of a thousand miles begins with a single step."
> — Lao Tzu
