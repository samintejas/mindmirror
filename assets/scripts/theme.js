// Theme toggle. The actual colors live in theme.css (CSS variables consumed by
// the Tailwind utilities), switched by the [data-theme] attribute on <html>
// (set pre-paint in the page head).
// With no attribute the page follows the OS preference; clicking the
// toggle pins an explicit choice and remembers it.
(function () {
    const root = document.documentElement;
    const button = document.getElementById("theme-toggle");
    const system = window.matchMedia("(prefers-color-scheme: dark)");

    function isDark() {
        const pinned = root.getAttribute("data-theme");
        return pinned ? pinned === "dark" : system.matches;
    }

    function paintIcon() {
        if (button) button.textContent = isDark() ? "☀" : "☾";
    }

    paintIcon();

    if (button) {
        button.addEventListener("click", () => {
            const next = isDark() ? "light" : "dark";
            root.setAttribute("data-theme", next);
            localStorage.setItem("theme", next);
            paintIcon();
        });
    }

    // Follow the OS while no explicit choice is pinned.
    system.addEventListener("change", () => {
        if (!root.getAttribute("data-theme")) paintIcon();
    });
})();
