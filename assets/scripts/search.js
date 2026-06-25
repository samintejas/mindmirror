// Client-side note search. Fetches /api/search (title + tags + description per
// page) and filters the file tree as you type, hiding non-matching files and
// collapsing folders with no visible descendants.
document.addEventListener("DOMContentLoaded", function () {
    const input = document.getElementById("search");
    const tree = document.getElementById("file-tree");
    if (!input || !tree) return;

    let index = {};
    fetch("/api/search")
        .then((r) => (r.ok ? r.json() : []))
        .then((data) => {
            data.forEach((e) => {
                index[e.href] =
                    (e.title + " " + (e.tags || []).join(" ") + " " + (e.description || ""))
                        .toLowerCase();
            });
        })
        .catch(() => {});

    input.addEventListener("input", function () {
        const q = input.value.trim().toLowerCase();

        tree.querySelectorAll("li.html-file").forEach((li) => {
            const a = li.querySelector("a");
            const href = a ? a.getAttribute("href") : "";
            const hay = index[href] || (a ? a.textContent.toLowerCase() : "");
            li.style.display = q === "" || hay.includes(q) ? "" : "none";
        });

        // Show a folder only if it has a visible descendant file; expand matches.
        tree.querySelectorAll("span.folder").forEach((span) => {
            const li = span.closest("li");
            const nested = span.nextElementSibling;
            if (!li) return;
            if (q === "") {
                li.style.display = "";
                return;
            }
            let visible = false;
            if (nested) {
                nested.querySelectorAll("li.html-file").forEach((f) => {
                    if (f.style.display !== "none") visible = true;
                });
            }
            li.style.display = visible ? "" : "none";
            if (visible && nested) {
                nested.classList.add("active");
                span.classList.add("active");
            }
        });
    });
});
