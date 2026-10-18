const routes = {
  compare: {
    tier: "T1",
    name: "Deterministic utility",
    description:
      "A strict pixel comparison has a mechanical output. Any aesthetic judgment raises the task floor.",
    role: "Design quality assurance",
    tools: "Odiff comparison only",
    check: "Compare two saved images",
  },
  code: {
    tier: "T2",
    name: "Structured synthesis",
    description:
      "Structured implementation needs synthesis. Debugging or architectural ambiguity raises the floor to T3.",
    role: "Backend engineering",
    tools: "Workspace read, list and write",
    check: "Read back the exact changed artifacts",
  },
  design: {
    tier: "T3",
    name: "Creative & architectural",
    description:
      "Layout decisions and multi-step visual reasoning require the architectural tier.",
    role: "Frontend engineering + interface design",
    tools: "Workspace, Playwright, image inspection",
    check: "Render the screen and inspect its pixels",
  },
};
for (const button of document.querySelectorAll("[data-route]"))
  button.addEventListener("click", () => {
    for (const other of document.querySelectorAll("[data-route]"))
      other.setAttribute("aria-pressed", String(other === button));
    for (const [key, value] of Object.entries(routes[button.dataset.route]))
      document.getElementById(`route-${key}`).textContent = value;
  });
const platforms = {
  mac: [
    "aarch64-apple-darwin",
    "macOS",
    "Rename the file to mirket, make it executable, and add its directory to PATH.",
  ],
  linux: [
    "x86_64-unknown-linux-gnu",
    "Linux",
    "Rename the file to mirket, make it executable, and add its directory to PATH.",
  ],
  windows: [
    "x86_64-pc-windows-msvc.exe",
    "Windows",
    "Rename the file to mirket.exe and add its directory to your user PATH.",
  ],
};
document.getElementById("platform").addEventListener("change", (event) => {
  const [asset, label, hint] = platforms[event.target.value];
  const link = document.getElementById("download");
  link.href = `https://github.com/codemirket/harness/releases/latest/download/mirket-${asset}`;
  link.textContent = `Download for ${label} ↓`;
  document.getElementById("install-hint").textContent = hint;
});
document.getElementById("copy").addEventListener("click", async () => {
  const status = document.getElementById("copy-status");
  try {
    await navigator.clipboard.writeText(
      document.getElementById("commands").textContent,
    );
    status.textContent = "Setup commands copied.";
  } catch {
    status.textContent =
      "Copy unavailable. Select the two commands above to copy them.";
  }
});
