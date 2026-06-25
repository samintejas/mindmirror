// Add-source form: show only the fields that match the chosen kind.
document.addEventListener("DOMContentLoaded", function () {
    const kind = document.getElementById("kind-select");
    if (!kind) return;

    const local = document.querySelectorAll(".field-local");
    const git = document.querySelectorAll(".field-git");

    function sync() {
        const isGit = kind.value === "git";
        local.forEach((f) => (f.hidden = isGit));
        git.forEach((f) => (f.hidden = !isGit));
    }

    kind.addEventListener("change", sync);
    sync();

    // "Browse…" asks the server to open the desktop folder picker and fills in
    // the chosen absolute path. The text field still works if it's unavailable.
    const browse = document.getElementById("browse");
    const pathInput = document.getElementById("src-path");
    if (browse && pathInput) {
        browse.addEventListener("click", async function () {
            browse.disabled = true;
            const label = browse.textContent;
            browse.textContent = "Opening…";
            try {
                const res = await fetch("/api/pick-folder");
                if (res.status === 200) {
                    const data = await res.json();
                    if (data.path) pathInput.value = data.path;
                } else if (res.status === 501) {
                    pathInput.placeholder = "Picker unavailable — type the path";
                    pathInput.focus();
                }
                // 204 = cancelled: leave the field as-is.
            } catch (_) {
                // Network/other error: manual entry still works.
            } finally {
                browse.disabled = false;
                browse.textContent = label;
            }
        });
    }
});
