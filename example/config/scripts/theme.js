document.addEventListener("DOMContentLoaded", function () {
  const themeToggle = document.getElementById("theme-toggle");
  const prefersDarkScheme = window.matchMedia("(prefers-color-scheme: dark)");

  function setTheme(theme) {
    document.body.classList.toggle("dark-theme", theme === "dark");
    document.body.classList.toggle("light-theme", theme === "light");
  }

  // Set initial theme based on user's preference
  setTheme(prefersDarkScheme.matches ? "dark" : "light");

  themeToggle.addEventListener("click", () => {
    const currentTheme = document.body.classList.contains("dark-theme")
      ? "dark"
      : "light";
    const newTheme = currentTheme === "dark" ? "light" : "dark";
    setTheme(newTheme);
  });

  // Listen for changes to the user's preferred color scheme
  prefersDarkScheme.addListener((e) => setTheme(e.matches ? "dark" : "light"));
});
