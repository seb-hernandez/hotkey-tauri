const { invoke } = window.__TAURI__.tauri;

async function run() {
  await invoke("run");
}

window.addEventListener("DOMContentLoaded", () => {
  const runButton = document.querySelector(".run");
  runButton.onclick = run;
});
