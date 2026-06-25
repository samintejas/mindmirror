# Comprehensive Markdown Showcase

## Table of Contents
1. [Introduction](#introduction)
2. [Text Formatting](#text-formatting)
3. [Lists](#lists)
4. [Links](#links)
5. [Images](#images)
6. [Code](#code)
7. [Blockquotes](#blockquotes)
8. [Tables](#tables)
9. [Task Lists](#task-lists)
10. [Horizontal Rules](#horizontal-rules)
11. [Footnotes](#footnotes)
12. [Emoji](#emoji)

## Introduction

This document showcases various Markdown features and syntax elements. It's designed to help you test and visualize different Markdown capabilities.

## Text Formatting

You can make text **bold**, *italic*, or ***both***. You can also use ~~strikethrough~~ for deleted text.

Sometimes you need to use `inline code` within a sentence.

## Lists

### Unordered Lists
- Item 1
- Item 2
  - Subitem 2.1
  - Subitem 2.2
- Item 3

### Ordered Lists
1. First item
2. Second item
   1. Subitem 2.1
   2. Subitem 2.2
3. Third item

## Links

[Visit OpenAI](https://www.openai.com)

You can also use [reference-style links][ref-link].

[ref-link]: https://www.example.com

## Images

Here's an example of an inline image:

![Markdown Logo](https://markdown-here.com/img/icon256.png)

And here's a reference-style image:

![Reference Image][ref-image]

[ref-image]: https://upload.wikimedia.org/wikipedia/commons/4/48/Markdown-mark.svg

## Code

Inline code: `const greeting = "Hello, World!";`

Code block:
```python
def fibonacci(n):
    if n <= 1:
        return n
    else:
        return fibonacci(n-1) + fibonacci(n-2)

print(fibonacci(10))
```

## Blockquotes

> This is a blockquote.
>
> It can span multiple lines.
>> You can also nest blockquotes.

## Tables

| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Row 1, Col 1 | Row 1, Col 2 | Row 1, Col 3 |
| Row 2, Col 1 | Row 2, Col 2 | Row 2, Col 3 |
| Row 3, Col 1 | Row 3, Col 2 | Row 3, Col 3 |

You can align columns:

| Left-aligned | Center-aligned | Right-aligned |
|:-------------|:--------------:|---------------:|
| Col 1 | Col 2 | Col 3 |
| Left | Center | Right |

## Task Lists

- [x] Complete item
- [ ] Incomplete item
- [x] Another complete item
  - [ ] Nested incomplete item

## Horizontal Rules

---

Or

***

Or

___

## Footnotes

Here's a sentence with a footnote. [^1]

[^1]: This is the footnote content.

## Emoji

Markdown also supports emoji shortcodes! :smile: :rocket: :books:
