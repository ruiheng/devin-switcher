const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.shell;

const $ = (id) => document.getElementById(id);
const esc = (s) =>
  String(s ?? "").replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c])
  );

let loginId = null;
let loginName = "";
let loginTimer = null;

function toast(msg, isErr = false) {
  const t = $("toast");
  t.textContent = msg;
  t.className = "toast" + (isErr ? " err" : "");
  clearTimeout(t._h);
  t._h = setTimeout(() => t.classList.add("hidden"), 4000);
}

async function refresh() {
  try {
    const [status, profiles] = await Promise.all([
      invoke("get_status"),
      invoke("list_profiles"),
    ]);
    renderStatus(status);
    renderProfiles(profiles);
  } catch (e) {
    toast(String(e), true);
  }
}

function renderStatus(s) {
  const dot = $("currentDot");
  if (s.auth && s.auth.logged_in) {
    dot.classList.add("on");
    $("currentLabel").textContent =
      s.auth.name || s.auth.email || "Signed in";
    $("currentSub").textContent = [s.auth.email, s.auth.plan || s.auth.tier]
      .filter(Boolean)
      .join(" · ");
  } else {
    dot.classList.remove("on");
    $("currentLabel").textContent = "Not signed in";
    $("currentSub").textContent = s.devin_installed
      ? ""
      : "devin CLI not found";
  }
  $("currentQuota").innerHTML = quotaText(s.usage);
}

// Render a Usage object as compact quota chips: weekly/daily % left with
// reset dates, ACU used/limit, overage balance, plan end.
function quotaText(u) {
  if (!u) return "";
  const parts = [];
  const fmtDate = (unix) => {
    const d = new Date(unix * 1000);
    return `${d.getMonth() + 1}/${d.getDate()}`;
  };
  if (u.daily_left != null) {
    parts.push(
      `<span class="q">Daily ${u.daily_left}% left` +
        (u.daily_reset_unix ? ` · resets ${fmtDate(u.daily_reset_unix)}` : "") +
        `</span>`
    );
  }
  if (u.weekly_left != null) {
    parts.push(
      `<span class="q">Weekly ${u.weekly_left}% left` +
        (u.weekly_reset_unix ? ` · resets ${fmtDate(u.weekly_reset_unix)}` : "") +
        `</span>`
    );
  }
  if (u.acu_limit != null && u.acu_limit > 0) {
    parts.push(
      `<span class="q">${r2(u.acu_used || 0)} / ${r2(u.acu_limit)} ACU</span>`
    );
  }
  if (u.overage_micros != null && u.overage_micros > 0) {
    parts.push(`<span class="q">+$${(u.overage_micros / 1e6).toFixed(2)}</span>`);
  }
  if (u.plan_end) {
    parts.push(`<span class="q dim">cycle ends ${esc(u.plan_end.slice(0, 10))}</span>`);
  }
  return parts.join(" ");
}

const r2 = (f) => Math.round(f * 100) / 100;

function renderProfiles(profiles) {
  const el = $("profiles");
  if (!profiles.length) {
    el.innerHTML =
      '<p class="sub" style="text-align:center;margin:24px 0">No profiles yet.<br>Add your first account below.</p>';
    return;
  }
  el.innerHTML = "";
  for (const p of profiles) {
    const card = document.createElement("div");
    card.className = "profile" + (p.is_active ? " active" : "");
    card.innerHTML = `
      <div class="top">
        <span class="name">${esc(p.name)}</span>
        ${p.is_active ? '<span class="badge">ACTIVE</span>' : ""}
      </div>
      <div class="meta">${esc(
        [p.email || p.display_name || "unknown account", p.plan]
          .filter(Boolean)
          .join(" · ")
      )}</div>
      ${p.usage ? `<div class="quota">${quotaText(p.usage)}</div>` : ""}
      ${p.note ? `<div class="note">${esc(p.note)}</div>` : ""}
      <div class="actions">
        <button class="use" data-act="use">Switch</button>
        <button data-act="par">Run in parallel</button>
        <button data-act="ref" title="Refresh quota">↻</button>
        <button class="danger" data-act="del">Delete</button>
      </div>`;
    card.querySelector('[data-act="use"]').onclick = () => useProfile(p.name);
    card.querySelector('[data-act="par"]').onclick = () => parallel(p.name);
    card.querySelector('[data-act="ref"]').onclick = () => refreshUsage(p.name);
    card.querySelector('[data-act="del"]').onclick = () => del(p.name);
    el.appendChild(card);
  }
}

async function useProfile(name) {
  try {
    const r = await invoke("use_profile", { name });
    const who = r.auth.logged_in
      ? r.auth.name || r.auth.email || "account"
      : "account";
    const warn = $("warnBanner");
    if (r.restart_needed && r.restart_needed.length) {
      warn.textContent =
        "Switched to " +
        who +
        ". Restart these to use the new account: " +
        r.restart_needed.join(", ");
      warn.classList.remove("hidden");
    } else {
      warn.classList.add("hidden");
    }
    toast("Switched to " + who);
    refresh();
  } catch (e) {
    toast(String(e), true);
  }
}

async function parallel(name) {
  try {
    const r = await invoke("launch_parallel", { name, cwd: null });
    if (r.launched) {
      toast("Launched a parallel devin session as " + name);
    } else {
      toast("Run this in a terminal: " + r.command);
    }
  } catch (e) {
    toast(String(e), true);
  }
}

async function refreshUsage(name) {
  try {
    await invoke("refresh_usage", { name });
    refresh();
  } catch (e) {
    toast(String(e), true);
  }
}

async function del(name) {
  if (!confirm(`Delete profile "${name}"? Its saved credentials are removed.`))
    return;
  try {
    await invoke("delete_profile", { name });
    refresh();
  } catch (e) {
    toast(String(e), true);
  }
}

// ---- Add sheet ----

$("addBtn").onclick = () => $("sheet").classList.remove("hidden");
$("closeSheet").onclick = closeSheet;

function closeSheet() {
  $("sheet").classList.add("hidden");
  stopLogin();
}

for (const t of document.querySelectorAll(".tab")) {
  t.onclick = () => {
    document
      .querySelectorAll(".tab")
      .forEach((x) => x.classList.toggle("active", x === t));
    for (const pane of document.querySelectorAll(".tabpane"))
      pane.classList.add("hidden");
    $("tab-" + t.dataset.tab).classList.remove("hidden");
  };
}

$("saveCurrent").onclick = async () => {
  const name = $("currentName").value.trim();
  if (!name) return toast("Pick a profile name", true);
  try {
    await invoke("save_current", { name, note: "" });
    toast("Saved");
    closeSheet();
    refresh();
  } catch (e) {
    toast(String(e), true);
  }
};

$("saveToken").onclick = async () => {
  const name = $("tokenName").value.trim();
  const token = $("tokenValue").value.trim();
  if (!name || !token) return toast("Name and token required", true);
  try {
    const p = await invoke("add_token", { name, token });
    toast("Saved " + (p.email || p.name));
    closeSheet();
    refresh();
  } catch (e) {
    toast(String(e), true);
  }
};

$("startLogin").onclick = async () => {
  const name = $("loginName").value.trim();
  if (!name) return toast("Pick a profile name", true);
  try {
    const offer = await invoke("start_login");
    loginId = offer.id;
    loginName = name;
    $("loginUrl").value = offer.url;
    $("loginHint").textContent = offer.hint;
    $("startLogin").classList.add("hidden");
    $("loginWait").classList.remove("hidden");
    loginTimer = setInterval(pollLogin, 1200);
  } catch (e) {
    toast(String(e), true);
  }
};

$("openUrl").onclick = () => {
  const url = $("loginUrl").value;
  if (url) open(url).catch(() => {});
};

$("copyUrl").onclick = async () => {
  try {
    await navigator.clipboard.writeText($("loginUrl").value);
    toast("Link copied — open it in any browser or incognito window");
  } catch {
    $("loginUrl").select();
  }
};

$("cancelLogin").onclick = () => {
  if (loginId != null) invoke("cancel_login", { id: loginId }).catch(() => {});
  stopLogin();
};

function stopLogin() {
  loginId = null;
  clearInterval(loginTimer);
  $("startLogin").classList.remove("hidden");
  $("loginWait").classList.add("hidden");
}

async function pollLogin() {
  if (loginId == null) return;
  try {
    const r = await invoke("poll_login", {
      id: loginId,
      name: loginName || $("loginName").value.trim() || "account",
      note: "",
    });
    if (r.kind === "done") {
      toast("Signed in as " + (r.profile.email || r.profile.name));
      stopLogin();
      closeSheet();
      refresh();
    }
    // "waiting" → keep polling
  } catch (e) {
    toast(String(e), true);
    stopLogin();
  }
}

refresh();
setInterval(refresh, 5000);
