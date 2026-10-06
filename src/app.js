const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.shell;

const $ = (id) => document.getElementById(id);
const esc = (s) =>
  String(s ?? "").replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c])
  );

// ---------- i18n ----------

const I18N = {
  en: {
    currentTag: "CURRENT SIGN-IN",
    checking: "Checking…",
    notSignedIn: "Not signed in",
    cliNotFound: "devin CLI not found",
    signedIn: "Signed in",
    addAccount: "+ Add account",
    addTitle: "Add account",
    tabLogin: "Browser sign-in",
    tabCurrent: "Save current",
    tabToken: "Paste token",
    loginHint:
      "Sign in with a browser. The page asks which account to use — your existing devin.ai session won't auto-apply. Open the link in any browser or an incognito window.",
    loginName:
      "The profile is named after the account's email — rename it from the card afterwards.",
    startLogin: "Start sign-in",
    waiting: "Waiting for sign-in…",
    copy: "Copy",
    open: "Open",
    terminalInstead: "Or run in a terminal instead",
    cancel: "Cancel",
    currentHint:
      "Save the account the devin CLI is signed in as right now. It's named after the account's email automatically.",
    saveCurrent: "Save current sign-in",
    tokenHint:
      "Get a sign-in link, open it in any browser — the page shows a code after sign-in. Paste the code (or a devin-session-token$…) below.",
    getLink: "Get sign-in link",
    codeOrToken: "Code or token",
    saveToken: "Save",
    noProfiles: "No profiles yet.<br>Add your first account below.",
    unknown: "unknown account",
    swUse: "Switch",
    swPar: "Run in parallel",
    swRen: "Rename",
    swRef: "Refresh quota",
    swDel: "Delete",
    active: "ACTIVE",
    daily: "Daily",
    weekly: "Weekly",
    resets: "resets",
    used: "used",
    overage: "overage",
    planEnds: "plan ends",
    savedAs: (n) => `Saved as ${n}`,
    savedTok: (n) => `Saved ${n}`,
    signedInAs: (n) => `Signed in as ${n}`,
    switchedTo: (w) => `Switched to ${w}`,
    restartWarn: (w, ps) =>
      `Switched to ${w}. Restart these to use the new account: ${ps}`,
    launched: (n) => `Launched a parallel devin session as ${n}`,
    runThis: (c) => `Run this in a terminal: ${c}`,
    linkCopied: "Link copied — open it in any browser or incognito window",
    tokenReq: "Token required",
    delConfirm: (n) =>
      `Delete profile "${n}"? Its saved credentials are removed.`,
    renamePrompt: "Rename profile",
  },
  zh: {
    currentTag: "当前登录",
    checking: "检查中…",
    notSignedIn: "未登录",
    cliNotFound: "找不到 devin CLI",
    signedIn: "已登录",
    addAccount: "+ 添加账号",
    addTitle: "添加账号",
    tabLogin: "浏览器登录",
    tabCurrent: "保存当前",
    tabToken: "粘贴 token",
    loginHint:
      "在浏览器中登录。页面会强制让你选账号——已登录的 devin.ai 会话不会被自动复用。链接可以在任何浏览器或隐身窗口打开。",
    loginName: "profile 会用账号邮箱自动命名，之后可在卡片上改名。",
    startLogin: "开始登录",
    waiting: "等待登录…",
    copy: "复制",
    open: "打开",
    terminalInstead: "或者在终端里运行",
    cancel: "取消",
    currentHint:
      "保存 devin CLI 当前登录的账号。profile 会按邮箱自动命名。",
    saveCurrent: "保存当前登录",
    tokenHint:
      "生成登录链接，在任何浏览器打开——登录后页面会显示一个 code。把 code（或 devin-session-token$…）粘贴到下面。",
    getLink: "生成登录链接",
    codeOrToken: "Code 或 token",
    saveToken: "保存",
    noProfiles: "还没有 profile。<br>在下方添加第一个账号。",
    unknown: "未知账号",
    swUse: "切换",
    swPar: "并行运行",
    swRen: "改名",
    swRef: "刷新额度",
    swDel: "删除",
    active: "使用中",
    daily: "当天",
    weekly: "本周",
    resets: "重置",
    used: "已用",
    overage: "超额余额",
    planEnds: "周期结束",
    savedAs: (n) => `已保存为 ${n}`,
    savedTok: (n) => `已保存 ${n}`,
    signedInAs: (n) => `已登录 ${n}`,
    switchedTo: (w) => `已切换到 ${w}`,
    restartWarn: (w, ps) => `已切换到 ${w}。重启这些进程后生效：${ps}`,
    launched: (n) => `已用 ${n} 启动并行 devin 会话`,
    runThis: (c) => `在终端中运行：${c}`,
    linkCopied: "链接已复制——可在任何浏览器或隐身窗口打开",
    tokenReq: "需要 token",
    delConfirm: (n) => `删除 profile「${n}」？保存的凭据将被移除。`,
    renamePrompt: "重命名 profile",
  },
};

let lang =
  localStorage.getItem("dsw-lang") ||
  ((navigator.language || "").toLowerCase().startsWith("zh") ? "zh" : "en");

const t = (k, ...a) => {
  const v = I18N[lang][k] ?? I18N.en[k] ?? k;
  return typeof v === "function" ? v(...a) : v;
};

function applyI18n() {
  document.documentElement.lang = lang === "zh" ? "zh-CN" : "en";
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    el.textContent = t(el.dataset.i18n);
  });
  $("langBtn").textContent = lang === "zh" ? "EN" : "中";
  refresh();
}

$("langBtn").onclick = () => {
  lang = lang === "zh" ? "en" : "zh";
  localStorage.setItem("dsw-lang", lang);
  applyI18n();
};

// ---------- app ----------

let loginId = null;
let loginTimer = null;

function toast(msg, isErr = false) {
  const t2 = $("toast");
  t2.textContent = msg;
  t2.className = "toast" + (isErr ? " err" : "");
  clearTimeout(t2._h);
  t2._h = setTimeout(() => t2.classList.add("hidden"), 4000);
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
      s.auth.name || s.auth.email || t("signedIn");
    $("currentSub").textContent = [s.auth.email, s.auth.plan || s.auth.tier]
      .filter(Boolean)
      .join(" · ");
  } else {
    dot.classList.remove("on");
    $("currentLabel").textContent = t("notSignedIn");
    $("currentSub").textContent = s.devin_installed ? "" : t("cliNotFound");
  }
  $("currentQuota").innerHTML = quotaHtml(s.usage);
}

// Usage → labeled progress bars. Fill = % remaining for daily/weekly,
// % remaining for ACU. Green → amber → red as it drains.
function quotaHtml(u) {
  if (!u) return "";
  const bar = (pct) => {
    const p = Math.max(0, Math.min(100, pct));
    const c = p > 50 ? "ok" : p > 20 ? "warn" : "low";
    return `<div class="qbar"><i class="${c}" style="width:${p}%"></i></div>`;
  };
  const fmtDate = (unix) => {
    const d = new Date(unix * 1000);
    return `${d.getMonth() + 1}/${d.getDate()}`;
  };
  const row = (label, pct, val) =>
    `<div class="qrow"><span class="qlab">${label}</span>` +
    `${bar(pct)}<span class="qval">${val}</span></div>`;
  const rows = [];
  if (u.daily_left != null) {
    rows.push(
      row(
        t("daily"),
        u.daily_left,
        `${Math.round(u.daily_left)}%` +
          (u.daily_reset_unix
            ? ` · ${t("resets")} ${fmtDate(u.daily_reset_unix)}`
            : "")
      )
    );
  }
  if (u.weekly_left != null) {
    rows.push(
      row(
        t("weekly"),
        u.weekly_left,
        `${Math.round(u.weekly_left)}%` +
          (u.weekly_reset_unix
            ? ` · ${t("resets")} ${fmtDate(u.weekly_reset_unix)}`
            : "")
      )
    );
  }
  if (u.acu_limit != null && u.acu_limit > 0) {
    const used = u.acu_used || 0;
    const leftPct = ((u.acu_limit - used) / u.acu_limit) * 100;
    rows.push(
      row("ACU", leftPct, `${r2(used)} / ${r2(u.acu_limit)} ${t("used")}`)
    );
  }
  const foot = [];
  if (u.overage_micros != null && u.overage_micros > 0) {
    foot.push(`${t("overage")} +$${(u.overage_micros / 1e6).toFixed(2)}`);
  }
  if (u.plan_end) foot.push(`${t("planEnds")} ${esc(u.plan_end.slice(0, 10))}`);
  if (foot.length) rows.push(`<div class="qfoot">${foot.join(" · ")}</div>`);
  return rows.join("");
}

const r2 = (f) => Math.round(f * 100) / 100;

function renderProfiles(profiles) {
  const el = $("profiles");
  // The banner only earns its space when the live sign-in is NOT a saved
  // profile (signed out, or an unsaved account). When it matches a card,
  // the ACTIVE badge already says it — hide the banner instead.
  const active = profiles.find((p) => p.is_active);
  $("current").classList.toggle("hidden", !!active);
  if (!profiles.length) {
    el.innerHTML = `<p class="sub" style="text-align:center;margin:24px 0">${t("noProfiles")}</p>`;
    return;
  }
  el.innerHTML = "";
  for (const p of profiles) {
    const card = document.createElement("div");
    card.className = "profile" + (p.is_active ? " active" : "");
    card.innerHTML = `
      <div class="top">
        <span class="name">${esc(p.name)}</span>
        ${p.is_active ? `<span class="badge">${t("active")}</span>` : ""}
      </div>
      <div class="meta">${esc(
        [p.email || p.display_name || t("unknown"), p.plan]
          .filter(Boolean)
          .join(" · ")
      )}</div>
      ${p.usage ? `<div class="quota">${quotaHtml(p.usage)}</div>` : ""}
      ${p.note ? `<div class="note">${esc(p.note)}</div>` : ""}
      <div class="actions">
        <button class="use" data-act="use">${t("swUse")}</button>
        <button data-act="par">${t("swPar")}</button>
        <button data-act="ren">${t("swRen")}</button>
        <button data-act="ref" title="${t("swRef")}">↻</button>
        <button class="danger" data-act="del">${t("swDel")}</button>
      </div>`;
    card.querySelector('[data-act="use"]').onclick = () => useProfile(p.name);
    card.querySelector('[data-act="par"]').onclick = () => parallel(p.name);
    card.querySelector('[data-act="ren"]').onclick = () => rename(p.name);
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
      warn.textContent = t("restartWarn", who, r.restart_needed.join(", "));
      warn.classList.remove("hidden");
    } else {
      warn.classList.add("hidden");
    }
    toast(t("switchedTo", who));
    refresh();
  } catch (e) {
    toast(String(e), true);
  }
}

async function parallel(name) {
  try {
    const r = await invoke("launch_parallel", { name, cwd: null });
    if (r.launched) {
      toast(t("launched", name));
    } else {
      toast(t("runThis", r.command));
    }
  } catch (e) {
    toast(String(e), true);
  }
}

async function rename(name) {
  const to = prompt(t("renamePrompt"), name);
  if (to == null || to.trim() === "" || to.trim() === name) return;
  try {
    await invoke("rename_profile", { from: name, to: to.trim() });
    refresh();
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
  if (!confirm(t("delConfirm", name))) return;
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

for (const t2 of document.querySelectorAll(".tab")) {
  t2.onclick = () => {
    document
      .querySelectorAll(".tab")
      .forEach((x) => x.classList.toggle("active", x === t2));
    for (const pane of document.querySelectorAll(".tabpane"))
      pane.classList.add("hidden");
    $("tab-" + t2.dataset.tab).classList.remove("hidden");
  };
}

$("saveCurrent").onclick = async () => {
  try {
    const p = await invoke("save_current", { name: "", note: "" });
    toast(t("savedAs", p.name));
    closeSheet();
    refresh();
  } catch (e) {
    toast(String(e), true);
  }
};

$("tokenLink").onclick = async () => {
  try {
    $("tokenUrl").value = await invoke("manual_start");
    $("tokenLinkRow").classList.remove("hidden");
  } catch (e) {
    toast(String(e), true);
  }
};

$("openTokenUrl").onclick = () => {
  const url = $("tokenUrl").value;
  if (url) open(url).catch(() => {});
};

$("copyTokenUrl").onclick = async () => {
  try {
    await navigator.clipboard.writeText($("tokenUrl").value);
    toast(t("linkCopied"));
  } catch {
    $("tokenUrl").select();
  }
};

$("saveToken").onclick = async () => {
  const code = $("tokenValue").value.trim();
  if (!code) return toast(t("tokenReq"), true);
  try {
    const p = await invoke("manual_finish", { name: "", code });
    toast(t("savedTok", p.email || p.name));
    closeSheet();
    refresh();
  } catch (e) {
    toast(String(e), true);
  }
};

$("startLogin").onclick = async () => {
  try {
    const offer = await invoke("start_login");
    loginId = offer.id;
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
    toast(t("linkCopied"));
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
    const r = await invoke("poll_login", { id: loginId, name: "", note: "" });
    if (r.kind === "done") {
      toast(t("signedInAs", r.profile.email || r.profile.name));
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

applyI18n();
setInterval(refresh, 5000);
