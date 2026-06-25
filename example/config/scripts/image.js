const imageUrls = [
  "https://cdn.svgator.com/images/2021/10/solar-system-animation.svg",
];

document.addEventListener("DOMContentLoaded", () => {
  const imgElement = document.getElementById("randomImage");
  if (imgElement) {
    imgElement.src = imageUrls[Math.floor(Math.random() * imageUrls.length)];
  } else {
    console.error("Image element not found");
  }
});
