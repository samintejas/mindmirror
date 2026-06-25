# Mindmirror

**Mindmirror** is a minimal static-site generator for markdown notes, written in **Rust**. It
turns one or more folders (or git repositories) of Markdown into a themed, browsable website with a
file-tree index, search, math, and code highlighting — and a built-in web page to manage your
content sources.

### Features
- Render Markdown (GFM) to styled HTML
- **Multiple content sources**: local folders *and* remote git repositories, managed from the home page
- **Live reload**: `serve --watch` rebuilds on save and refreshes open browsers automatically
- **TOML frontmatter** (`+++`) for per-page layout, theme, and metadata (title, tags, draft, …)
- Inline and block math (MathJax) and code highlighting (highlight.js)
- Client-side search across your notes
- Bundled default themes/scripts/templates, all overridable from your config directory
- Incremental, content-hash builds (only changed files are regenerated)

## Installation

Build and install with Cargo (Rust 1.85+):

```sh
git clone https://github.com/samintejas/mindmirror.git
cd mindmirror
cargo install --path .
```

This installs the `mindmirror` binary (usually into `~/.cargo/bin`).

## Quick start

```sh
# 1. Scaffold a config file and copy the default themes/scripts into your config dir
mindmirror init

# 2. Register a content source (a folder of .md files)
mindmirror source add notes --path ~/notes

#    …or a remote git repo (cloned and tracked)
mindmirror source add blog --git https://github.com/you/blog.git --branch main

# 3. Serve with live reload and open http://localhost:8080
mindmirror serve --watch
```

The home page lists your sources with a file tree and lets you **add**, **resync** (re-pull a git
source), and **remove** sources without leaving the browser.

## Commands

| Command | Description |
|---|---|
| `mindmirror init` | Write a default `mindmirror.toml` and materialize default assets |
| `mindmirror build` | Build HTML for every registered source |
| `mindmirror serve [--port N] [--watch]` | Serve the site + source manager; `--watch` enables rebuild + live reload |
| `mindmirror clean` | Remove the build output directory |
| `mindmirror source list` | List registered sources |
| `mindmirror source add <name> --path <dir>` | Add a local source |
| `mindmirror source add <name> --git <url> [--branch <b>]` | Add a remote git source |
| `mindmirror source resync <name>` | Re-pull a git source (or re-validate a local one) and rebuild |
| `mindmirror source remove <name>` | Remove a source and its output |

> The file watcher applies to **local** sources only. Refresh git sources with the **Resync** button
> (or `mindmirror source resync`).

## Configuration

Config is TOML and fully optional — every field has a sensible default. Discovery order:
`--config <file>` → `$XDG_CONFIG_HOME/mindmirror/mindmirror.toml` → `/etc/mindmirror/` → current dir.

```toml
port = 8080

[output]
dest = "~/.cache/mindmirror/public"     # build output root; each source builds into dest/<name>

[styles]
# Optional override dir; embedded defaults are used when unset. Files in
# <config>/styles also override embedded ones.
# path = "~/.config/mindmirror/styles"
page_stylesheet  = "water-auto.css"     # default page theme; overridable per page

[scripts]
# path = "~/.config/mindmirror/scripts"

[defaults]
layout = "default"                      # default page layout/template
```

## Frontmatter

A page may start with a `+++`-fenced TOML block:

```markdown
+++
title  = "My Note"
layout = "wide"          # selects templates/wide.html (falls back to the default layout)
style  = "tufte.css"     # overrides the page stylesheet for this page
tags   = ["rust", "notes"]
draft  = false           # true → skipped during the build
description = "A short summary used by the index and search"
+++

# My Note
...
```

## Customizing themes, scripts, and templates

`mindmirror init` copies the embedded defaults into `$XDG_CONFIG_HOME/mindmirror/`
(`styles/`, `scripts/`, `templates/`). Edit those files to customize; anything present there
overrides the corresponding embedded default. Layouts are minijinja templates that receive
`title`, `body`, and `style`.

## Docker

```sh
docker build -t mindmirror .
docker run -p 8080:8080 -v ~/notes:/notes mindmirror
```
