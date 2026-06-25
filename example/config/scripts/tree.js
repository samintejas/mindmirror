document.addEventListener("DOMContentLoaded", function () {
  const folders = document.querySelectorAll(".folder");

  folders.forEach((folder) => {
    folder.addEventListener("click", function (event) {
      event.stopPropagation();
      toggleFolder(this);
    });

    folder.addEventListener("keydown", function (event) {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        toggleFolder(this);
      }
    });
  });

  function toggleFolder(folder) {
    folder.classList.toggle("active");
    const nestedItem = folder.nextElementSibling;
    if (nestedItem && nestedItem.classList.contains("nested-item")) {
      nestedItem.classList.toggle("active");
    }
  }

  // Add mobile sidebar toggle functionality
  const indexToggle = document.querySelector("#nav h2");
  const fileTree = document.getElementById("file-tree");

  indexToggle.addEventListener("click", function () {
    fileTree.classList.toggle("active");
  });
});
