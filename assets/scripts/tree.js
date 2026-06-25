// Collapsible folders in the file tree. A folder toggles the sibling
// <ul class="nested-item"> that holds its children.
document.addEventListener("DOMContentLoaded", function () {
    function toggle(folder) {
        folder.classList.toggle("active");
        const nested = folder.nextElementSibling;
        if (nested && nested.classList.contains("nested-item")) {
            nested.classList.toggle("active");
        }
    }

    document.querySelectorAll(".folder").forEach((folder) => {
        folder.setAttribute("tabindex", "0");
        folder.setAttribute("role", "button");

        folder.addEventListener("click", function (event) {
            event.stopPropagation();
            toggle(this);
        });

        folder.addEventListener("keydown", function (event) {
            if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                toggle(this);
            }
        });
    });
});
