const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const appWindow = window.__TAURI__.window.getCurrentWindow();
const webview = window.__TAURI__.webview.getCurrentWebview();

const $ = (id) => document.getElementById(id);
const ICON = { Phone: "\uE8EA", Tablet: "\uE70A", Laptop: "\uE7F8", Unknown: "\uE8EA" };

// Radar slots in % of the radar width: inner ring first, then the outer one.
const SLOTS = [
  [-90, 34], [-30, 34], [-150, 34], [30, 34], [150, 34], [90, 34],
  [-60, 44], [-120, 44], [0, 44], [180, 44], [60, 44], [120, 44],
].map(([deg, r]) => [Math.cos((deg * Math.PI) / 180) * r, Math.sin((deg * Math.PI) / 180) * r]);

const devices = new Map(); // id -> { info, el, slot }
const transfers = new Map(); // transfer id -> { el, rtype, name }
let files = [];
let helpTimer;
let dialogId = null;

const fmtBytes = (n) => {
  if (!n) return "0 B";
  const u = ["B", "KB", "MB", "GB"];
  const i = Math.min(u.length - 1, Math.floor(Math.log(n) / Math.log(1024)));
  return `${(n / 1024 ** i).toLocaleString("it-IT", { maximumFractionDigits: i ? 1 : 0 })} ${u[i]}`;
};
const baseName = (p) => p.split(/[\\/]/).pop();
const dirName = (p) => p.split(/[\\/]/).slice(0, -1).join("\\");

// What failed, and what the person can do about it. The raw backend text is kept as a detail.
const ERRORS = {
  start_discovery: (raw) => [
    "Impossibile avviare la ricerca dei dispositivi.",
    /in use|10048/i.test(raw)
      ? "Un'altra app sta già usando Quick Share (l'app di Google o un'altra copia di Scambio). Chiudila e riapri Scambio."
      : "Controlla di essere connesso a una rete Wi-Fi, poi riapri Scambio.",
  ],
  send_files: () => ["Impossibile avviare l'invio.", "Riprova. Se il problema continua, riapri Scambio."],
  pick_files: () => ["Impossibile aprire la scelta dei file.", "Riprova."],
  pick_download_dir: () => ["Impossibile cambiare la cartella.", "Riprova scegliendo una cartella in cui puoi scrivere."],
  open_download_dir: () => ["Impossibile aprire la cartella.", "Controlla che esista ancora o scegline un'altra con Cambia."],
};

function showError(cmd, raw) {
  const [title, fix] = (ERRORS[cmd] || (() => ["Operazione non riuscita.", "Riprova."]))(raw);
  $("error-title").textContent = title;
  $("error-text").textContent = fix;
  $("error-detail").textContent = raw ? `Dettagli: ${raw}` : "";
  $("error").hidden = false;
}

async function call(cmd, args) {
  try {
    return await invoke(cmd, args);
  } catch (e) {
    showError(cmd, String(e));
    throw e;
  }
}

/* ---------- Radar ---------- */

function updateRadarStatus() {
  const n = devices.size;
  $("radar-status").textContent = n
    ? `${n} ${n === 1 ? "dispositivo trovato" : "dispositivi trovati"}. Continuo a cercare…`
    : "Cerco dispositivi nelle vicinanze…";
  $("help").hidden = n > 0 || !helpShown;
}

let helpShown = false;
function armHelp() {
  clearTimeout(helpTimer);
  helpTimer = setTimeout(() => {
    helpShown = true;
    updateRadarStatus();
  }, 15000);
}

function addDevice(info) {
  if (devices.has(info.id)) return;
  const used = new Set([...devices.values()].map((d) => d.slot));
  const slot = SLOTS.findIndex((_, i) => !used.has(i));
  if (slot < 0) return; // radar full; more than 12 phones around is not a real case

  const el = $("tpl-device").content.firstElementChild.cloneNode(true);
  const [x, y] = SLOTS[slot];
  el.style.setProperty("--x", x);
  el.style.setProperty("--y", y);
  el.querySelector(".ic").textContent = ICON[info.rtype] || ICON.Unknown;
  el.querySelector(".device-name").textContent = info.name;
  const btn = el.querySelector(".device-btn");
  btn.title = `Invia a ${info.name}`;
  btn.setAttribute("aria-label", `Invia file a ${info.name}`);
  btn.addEventListener("click", () => sendTo(info));
  $("devices").append(el);
  devices.set(info.id, { info, el, slot });
  updateRadarStatus();
}

function removeDevice(id) {
  const d = devices.get(id);
  if (!d) return;
  devices.delete(id);
  d.el.classList.add("leaving");
  d.el.addEventListener("animationend", () => d.el.remove(), { once: true });
  updateRadarStatus();
}

function setDeviceBusy(id, busy, progress) {
  const el = devices.get(id)?.el;
  if (!el) return;
  el.classList.toggle("busy", busy);
  el.classList.toggle("has-progress", busy && progress !== undefined);
  if (progress !== undefined) el.style.setProperty("--p", progress);
}

listen("device", ({ payload }) => {
  if (payload.name) addDevice(payload);
  else removeDevice(payload.id);
});

/* ---------- Files ---------- */

function renderFiles() {
  const list = $("file-list");
  list.replaceChildren(
    ...files.map((path, i) => {
      const row = $("tpl-file").content.firstElementChild.cloneNode(true);
      row.querySelector(".file-name").textContent = baseName(path);
      row.querySelector(".file-dir").textContent = dirName(path);
      row.title = path;
      const rm = row.querySelector(".file-remove");
      rm.setAttribute("aria-label", `Rimuovi ${baseName(path)}`);
      rm.addEventListener("click", () => {
        files.splice(i, 1);
        renderFiles();
      });
      return row;
    }),
  );
  const has = files.length > 0;
  $("drop-empty").hidden = has;
  list.hidden = !has;
  $("files-foot").hidden = !has;
  $("files-clear").hidden = !has;
  $("files-card").classList.toggle("armed", has);
  $("files-count").textContent = `${files.length} file ${files.length === 1 ? "pronto" : "pronti"}`;
}

function addFiles(paths) {
  for (const p of paths) if (!files.includes(p)) files.push(p);
  renderFiles();
}

async function pickFiles() {
  const picked = await call("pick_files");
  addFiles(picked);
  return picked.length > 0;
}

$("pick").addEventListener("click", pickFiles);
$("pick-more").addEventListener("click", pickFiles);
$("files-clear").addEventListener("click", () => {
  files = [];
  renderFiles();
});

webview.onDragDropEvent(({ payload }) => {
  const overlay = $("drop-overlay");
  if (payload.type === "enter" || payload.type === "over") overlay.hidden = false;
  else overlay.hidden = true;
  if (payload.type === "drop" && payload.paths?.length) addFiles(payload.paths);
});

async function sendTo(info) {
  if (!files.length && !(await pickFiles())) return;
  const batch = [...files];
  transfers.set(info.id, { rtype: "Outbound", name: info.name, files: batch });
  renderTransfer({ id: info.id, rtype: "Outbound", state: "Initial", meta: null });
  setDeviceBusy(info.id, true);
  try {
    await call("send_files", { id: info.id, name: info.name, addr: `${info.ip}:${info.port}`, files: batch });
    files = [];
    renderFiles();
  } catch {
    finishTransfer(info.id, "fail", "Impossibile avviare l'invio");
  }
}

/* ---------- Transfers ---------- */

const ICONS = { out: "\uE898", in: "\uE896", ok: "\uE930", fail: "\uEA39" };

function transferEl(id, rtype) {
  let t = transfers.get(id);
  if (!t) {
    t = { rtype };
    transfers.set(id, t);
  }
  if (!t.el) {
    t.el = $("tpl-transfer").content.firstElementChild.cloneNode(true);
    $("transfers").prepend(t.el);
    $("transfers-wrap").hidden = false;
    t.el.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }
  return t;
}

function setActions(t, actions) {
  const box = t.el.querySelector(".transfer-actions");
  box.replaceChildren(
    ...actions.map(([label, fn, cls = "btn", icon]) => {
      const b = document.createElement("button");
      b.className = cls;
      if (icon) {
        b.innerHTML = `<i class="ic">${icon}</i>`;
        b.setAttribute("aria-label", label);
        b.title = label;
      } else b.textContent = label;
      b.addEventListener("click", fn);
      return b;
    }),
  );
}

function dismiss(id) {
  const t = transfers.get(id);
  t?.el?.remove();
  transfers.delete(id);
  $("transfers-wrap").hidden = !$("transfers").children.length;
}

function setStatus(t, text, pin) {
  const s = t.el.querySelector(".transfer-status");
  s.textContent = text;
  if (pin) {
    const p = document.createElement("span");
    p.className = "pin-inline";
    p.textContent = pin;
    s.append(" · PIN ", p);
  }
}

function setProgress(t, mode, value = 0) {
  const bar = t.el.querySelector(".progress");
  bar.hidden = mode === "none";
  bar.classList.toggle("indeterminate", mode === "indeterminate");
  bar.querySelector(".progress-fill").style.setProperty("--p", value);
}

function finishTransfer(id, kind, text) {
  const t = transferEl(id, transfers.get(id)?.rtype || "Outbound");
  t.done = true;
  t.el.classList.remove("ok", "fail");
  t.el.classList.add(kind);
  t.el.querySelector(".transfer-icon").textContent = ICONS[kind];
  setStatus(t, text);
  setProgress(t, "none");
  const actions = [];
  if (kind === "ok" && t.rtype === "Inbound" && !t.text) actions.push(["Apri cartella", () => invoke("open_download_dir")]);
  if (t.text) actions.push(["Copia", () => navigator.clipboard.writeText(t.text)]);
  actions.push(["Rimuovi dall'elenco", () => dismiss(id), "btn subtle icon-only", "\uE711"]);
  setActions(t, actions);
  setDeviceBusy(id, false);
}

function renderTransfer(m) {
  const meta = m.meta || {};
  const inbound = m.rtype === "Inbound";
  const t = transferEl(m.id, m.rtype);
  if (t.done && m.state !== "Initial") return;
  if (m.state === "Initial") {
    t.done = false;
    t.el.classList.remove("ok", "fail");
  }
  const known = inbound ? meta.source?.name : t.name || devices.get(m.id)?.info.name || meta.source?.name;
  const name = known || "un dispositivo";
  const fileList = meta.files || t.files || [];
  const what = meta.text_description
    ? meta.text_description
    : fileList.length === 1
      ? baseName(fileList[0])
      : fileList.length
        ? `${fileList.length} file`
        : "";

  t.el.querySelector(".transfer-icon").textContent = inbound ? ICONS.in : ICONS.out;
  t.el.querySelector(".transfer-title").textContent = `${inbound ? "Da" : "A"} ${name}${what ? ` · ${what}` : ""}`.replace(/^(Da|A) un dispositivo · /, "");

  const cancel = ["Annulla", () => invoke("respond", { id: m.id, action: "CancelTransfer" }), "btn subtle"];
  const pct = meta.total_bytes ? meta.ack_bytes / meta.total_bytes : 0;

  switch (m.state) {
    case "WaitingForUserConsent":
      if (inbound) {
        setStatus(t, "In attesa della tua conferma", meta.pin_code);
        setProgress(t, "none");
        setActions(t, [["Rispondi", () => openDialog(m)]]);
        openDialog(m);
      } else {
        setStatus(t, "Accetta sul telefono", meta.pin_code);
        setProgress(t, "indeterminate");
        setActions(t, [cancel]);
      }
      break;
    case "SendingFiles":
    case "ReceivingFiles":
      if (dialogId === m.id) closeDialog();
      setStatus(t, `${inbound ? "Ricezione" : "Invio"} · ${fmtBytes(meta.ack_bytes)} di ${fmtBytes(meta.total_bytes)}`);
      setProgress(t, "value", pct);
      setActions(t, [cancel]);
      if (!inbound) setDeviceBusy(m.id, true, pct);
      break;
    case "Finished":
      if (meta.text_payload) t.text = meta.text_payload;
      finishTransfer(m.id, "ok", inbound ? (t.text ? t.text : `Ricevuto in ${meta.destination || "Download"}`) : "Inviato");
      if (t.text) {
        t.el.querySelector(".transfer-status").textContent = meta.text_type === "Url" ? "Link ricevuto" : "Testo ricevuto";
        const box = document.createElement("div");
        box.className = "transfer-text-payload";
        box.textContent = t.text;
        t.el.append(box);
      }
      break;
    case "Rejected":
      finishTransfer(m.id, "fail", inbound || !known ? "Rifiutato" : `${known} ha rifiutato`);
      break;
    case "Cancelled":
      finishTransfer(m.id, "fail", "Annullato");
      break;
    case "Disconnected":
      finishTransfer(m.id, "fail", "Connessione interrotta. Riprova sulla stessa rete Wi-Fi.");
      break;
    default:
      setStatus(t, inbound ? "Connessione in corso…" : `Connessione a ${name}…`, meta.pin_code);
      setProgress(t, "indeterminate");
      setActions(t, inbound ? [] : [cancel]);
  }
  if (dialogId === m.id && m.state !== "WaitingForUserConsent") closeDialog();
}

listen("transfer", ({ payload }) => renderTransfer(payload));

/* ---------- Incoming request dialog ---------- */

function openDialog(m) {
  const meta = m.meta || {};
  dialogId = m.id;
  const name = meta.source?.name || "Un dispositivo";
  const n = meta.files?.length || 0;
  $("dlg-title").textContent = meta.text_description
    ? `${name} vuole inviarti ${meta.text_type === "Url" ? "un link" : "un testo"}`
    : `${name} vuole inviarti ${n} file`;
  const list = $("dlg-files");
  if (meta.text_description) {
    $("dlg-desc").textContent = "";
    list.replaceChildren(Object.assign(document.createElement("li"), { textContent: meta.text_description }));
  } else {
    $("dlg-desc").textContent = fmtBytes(meta.total_bytes);
    list.replaceChildren(...(meta.files || []).map((f) => Object.assign(document.createElement("li"), { textContent: f })));
  }
  $("dlg-pin").textContent = meta.pin_code || "—";
  $("dialog-smoke").hidden = false;
  $("dlg-accept").focus();
}

function closeDialog() {
  dialogId = null;
  $("dialog-smoke").hidden = true;
}

function answer(action) {
  if (!dialogId) return;
  invoke("respond", { id: dialogId, action });
  closeDialog();
}

$("dlg-accept").addEventListener("click", () => answer("AcceptTransfer"));
$("dlg-reject").addEventListener("click", () => answer("RejectTransfer"));
document.addEventListener("keydown", (e) => {
  if ($("dialog-smoke").hidden) return;
  if (e.key === "Escape") answer("RejectTransfer");
  if (e.key === "Tab") {
    // Keep focus inside the dialog
    const a = $("dlg-accept"), r = $("dlg-reject");
    if (e.shiftKey && document.activeElement === a) { r.focus(); e.preventDefault(); }
    else if (!e.shiftKey && document.activeElement === r) { a.focus(); e.preventDefault(); }
  }
});

/* ---------- Receiving settings ---------- */

$("visible").addEventListener("change", (e) => {
  const on = e.target.checked;
  call("set_visible", { visible: on });
  document.querySelector(".toggle-label").textContent = on ? "Attivato" : "Disattivato";
  $("visible-desc").textContent = on
    ? "I telefoni vicini possono inviarti file. Dovrai sempre accettare."
    : "Il PC non è visibile: puoi solo inviare.";
});
$("open-dir").addEventListener("click", () => call("open_download_dir"));
$("change-dir").addEventListener("click", async () => {
  const dir = await call("pick_download_dir");
  if (dir) setDir(dir);
});

function setDir(dir) {
  $("download-dir").textContent = dir;
  $("download-dir").title = dir;
}

/* ---------- Window chrome ---------- */

$("btn-min").addEventListener("click", () => appWindow.minimize());
$("btn-close").addEventListener("click", () => appWindow.close());
$("help-close").addEventListener("click", () => { helpShown = false; $("help").hidden = true; });
$("error-close").addEventListener("click", () => { $("error").hidden = true; });
document.addEventListener("contextmenu", (e) => {
  if (!e.target.closest(".transfer-text-payload")) e.preventDefault();
});

/* ---------- Start ---------- */

(async () => {
  const info = await call("info");
  document.documentElement.classList.toggle("mica", info.mica);
  $("me-name").textContent = info.name;
  $("me-name").title = `I telefoni vedono questo PC come ${info.name}`;
  setDir(info.download_dir);
  await call("start_discovery");
  armHelp();
  updateRadarStatus();
})();
