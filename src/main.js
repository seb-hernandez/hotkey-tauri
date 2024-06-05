const {invoke} = window.__TAURI__.tauri;

async function run() {
    document.getElementById("run").disabled = true;
    await invoke("run");
    document.getElementById("run").disabled = false;
}

window.addEventListener("DOMContentLoaded", () => {
    document.getElementById("run").onclick = run;
});
