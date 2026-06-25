// Tailwind Play CDN configuration (offline build of tailwind.js).
//
// Colors and fonts are mapped to CSS custom properties so the live theme is
// driven entirely by the linked stylesheet (light by default, auto-dark via
// `prefers-color-scheme`, pinned via the `data-theme` attribute on <html>).
// Because of that, no `dark:` variants are needed — the variables switch and
// every utility (and `prose`) follows. Fallbacks keep things sane on legacy
// theme stylesheets that don't define the variables.
tailwind.config = {
    theme: {
        extend: {
            colors: {
                paper: "var(--bg, #f6f5f3)",
                surface: "var(--surface, #ffffff)",
                "surface-2": "var(--surface-2, #fbfbfa)",
                line: "var(--border, #e8e6e1)",
                ink: "var(--ink, #1b1a20)",
                muted: "var(--muted, #6c6a78)",
                iris: "var(--accent, #5b54e8)",
                "iris-soft": "var(--accent-soft, #ecebfb)",
                danger: "var(--danger, #c8402f)",
            },
            fontFamily: {
                sans: ["var(--sans)", "system-ui", "sans-serif"],
                mono: ["var(--mono)", "ui-monospace", "monospace"],
            },
            boxShadow: {
                soft: "var(--shadow, 0 1px 2px rgba(20,18,40,.04), 0 8px 24px rgba(20,18,40,.05))",
            },
            maxWidth: {
                col: "var(--col, 760px)",
            },
            typography: {
                DEFAULT: {
                    css: {
                        "--tw-prose-body": "var(--ink, #1b1a20)",
                        "--tw-prose-headings": "var(--ink, #1b1a20)",
                        "--tw-prose-lead": "var(--muted, #6c6a78)",
                        "--tw-prose-links": "var(--accent, #5b54e8)",
                        "--tw-prose-bold": "var(--ink, #1b1a20)",
                        "--tw-prose-counters": "var(--muted, #6c6a78)",
                        "--tw-prose-bullets": "var(--accent, #5b54e8)",
                        "--tw-prose-hr": "var(--border, #e8e6e1)",
                        "--tw-prose-quotes": "var(--ink, #1b1a20)",
                        "--tw-prose-quote-borders": "var(--accent, #5b54e8)",
                        "--tw-prose-captions": "var(--muted, #6c6a78)",
                        "--tw-prose-code": "var(--ink, #1b1a20)",
                        "--tw-prose-pre-code": "var(--code-ink, #e9e8ef)",
                        "--tw-prose-pre-bg": "var(--code-bg, #1c1c23)",
                        "--tw-prose-th-borders": "var(--border, #e8e6e1)",
                        "--tw-prose-td-borders": "var(--border, #e8e6e1)",
                        maxWidth: "none",
                        // Let highlight.js own the code-block palette; prose just
                        // provides the frame.
                        "pre code": { backgroundColor: "transparent", color: "inherit" },
                        a: {
                            textDecoration: "none",
                            fontWeight: "500",
                            borderBottom: "1px solid var(--accent-soft, #ecebfb)",
                            transition: "border-color .15s",
                        },
                        "a:hover": { borderBottomColor: "var(--accent, #5b54e8)" },
                        "code::before": { content: "none" },
                        "code::after": { content: "none" },
                    },
                },
            },
        },
    },
};
