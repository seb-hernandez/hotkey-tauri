const {invoke} = window.__TAURI__.tauri;

async function run() {
    document.getElementById("run").disabled = true;
    await invoke("run");
    document.getElementById("run").disabled = false;
}

async function acc() {
    await invoke("acc");
}

window.addEventListener("DOMContentLoaded", () => {
    document.getElementById("run").onclick = run;
    document.getElementById("acc").onclick = acc;
});
