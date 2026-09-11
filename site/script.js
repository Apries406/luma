const dialog = document.querySelector(".demo-dialog");
const code = document.querySelector("#demo-code");
const output = document.querySelector(".demo-output");

for (const trigger of document.querySelectorAll("[data-open-demo]")) {
  trigger.addEventListener("click", () => dialog.showModal());
}

document.querySelector("[data-close-demo]").addEventListener("click", () => dialog.close());

dialog.addEventListener("click", (event) => {
  if (event.target === dialog) dialog.close();
});

if (location.hash === "#demo") dialog.showModal();

window.addEventListener("hashchange", () => {
  if (location.hash === "#demo" && !dialog.open) dialog.showModal();
});

document.querySelector("[data-run-demo]").addEventListener("click", () => {
  const hasMain = /\bmain\s*\(/.test(code.value);
  const hasPrintf = /\bprintf\s*\(/.test(code.value);

  if (!hasMain) {
    output.textContent = "Build stopped: add a main function first.";
    return;
  }

  output.textContent = hasPrintf
    ? "✓ Preview check passed\nDesktop Luma would now build and run this file."
    : "✓ main function found\nAdd printf to see terminal output in desktop Luma.";
});
