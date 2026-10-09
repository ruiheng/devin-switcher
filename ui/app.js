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
    cliTag: "CLI",
    desktopTag: "DESKTOP",
    bothTag: "CLI+DESKTOP",
    activeDesk: "DESKTOP",
    checking: "Checking…",
    notSignedIn: "Not signed in",
    cliNotFound: "devin CLI not found",
    signedIn: "Signed in",
    addAccount: "+ Add account",
    addTitle: "Add account",
    tabLogin: "Browser sign-in",
    tabCurrent: "Save current",
    tabToken: "Paste token",
    tabDesktop: "Desktop import",
    desktopHint:
      "Import the account Devin Desktop is signed in as — reads its stored session; Desktop can stay open.",
    importDesktop: "Import Desktop sign-in",
    importing: "Reading Devin Desktop's sign-in…",
    saving: "Saving…",
    loginHint:
      "Sign in via your browser — it uses the devin account already signed in there, if any.",
    loginName:
      "The account is named after its email — rename it from the card afterwards.",
    startLogin: "Start sign-in",
    waiting: "Waiting for sign-in…",
    finishing: "Callback received — finishing sign-in…",
    copy: "Copy",
    open: "Open",
    terminalInstead: "Or run in a terminal instead",
    cancel: "Cancel",
    logoutHint:
      "To sign in with a different devin account, sign out in your browser first:",
    openLogout: "Sign out in default browser",
    copyLogout: "Copy sign-out link",
    currentHint:
      "Save the account the devin CLI is signed in as right now. It's named after the account's email automatically.",
    saveCurrent: "Save current sign-in",
    tokenHint:
      "Get a sign-in link, open it in any browser — the page shows a code after sign-in. Paste the code (or a devin-session-token$…) below.",
    getLink: "Get sign-in link",
    codeOrToken: "Code or token",
    codeOrTokenPh: "code or devin-session-token$…",
    saveToken: "Save",
    savedAccounts: "SAVED ACCOUNTS",
    noProfiles: "No saved accounts yet.<br>Use + Add account above.",
    noProfilesSignedIn:
      "No saved accounts yet — the current sign-in isn't saved.",
    unknown: "unknown account",
    swUse: "Switch",
    swPar: "Run in parallel",
    swHint: "double-click to switch",
    renHint: "double-click to rename",
    swRef: "Refresh quota",
    swDel: "Delete",
    switching: (n) => `Switching to ${n}…`,
    active: "ACTIVE",
    daily: "Daily",
    weekly: "Weekly",
    resetNow: "resetting shortly",
    resetInHMS: (c) => `resets in ${c}`,
    resetInDH: (d, h) => `resets in ${d}d ${h}h`,
    resetOn: (d) => `resets ${d}`,
    used: "used",
    overage: "overage",
    planEnds: "plan ends",
    savedAs: (n) => `Saved as ${n}`,
    savedTok: (n) => `Saved ${n}`,
    signedInAs: (n) => `Signed in as ${n}`,
    refAll: "Refresh all",
    copyHint: "Click to copy",
    copied: "Copied",
    switchedTo: (w) => `Switched to ${w}`,
    swScopeCli: "Switch CLI only",
    swScopeDesktop: "Switch Desktop only",
    swMore: "More switch options",
    swToastBoth: (n) => `Switched to ${n} (CLI + Desktop)`,
    swToastCli: (n) => `Switched to ${n} (CLI only)`,
    swToastDesktop: (n) => `Switched Devin Desktop to ${n}`,
    swToastNoDesktop: (n) =>
      `Switched to ${n} (CLI; Devin Desktop not installed)`,
    swToastDesktopFail: (n, e) =>
      `Switched to ${n} (CLI) — Desktop sync failed: ${e}`,
    swQuitDesktop:
      "Devin Desktop is running — quit it fully, then switch again",
    restartWarn: (w, ps) =>
      `Switched to ${w}. Restart these to use the new account: ${ps}`,
    launched: (n) => `Launched a parallel devin session as ${n}`,
    runThis: (c) => `Run this in a terminal: ${c}`,
    linkCopied: "Link copied — open it in any browser or incognito window",
    tokenReq: "Token required",
    delSure: "Confirm delete",
    delSureLive: "Snapshot only — stays signed in",
  },
  zh: {
    currentTag: "当前登录",
    cliTag: "CLI",
    desktopTag: "桌面端",
    bothTag: "CLI+桌面端",
    activeDesk: "桌面端",
    checking: "检查中…",
    notSignedIn: "未登录",
    cliNotFound: "找不到 devin CLI",
    signedIn: "已登录",
    addAccount: "+ 添加账号",
    addTitle: "添加账号",
    tabLogin: "浏览器登录",
    tabCurrent: "保存当前",
    tabToken: "粘贴令牌",
    tabDesktop: "桌面端导入",
    desktopHint:
      "导入 Devin 桌面端当前登录的账号——直接读取它存的会话，桌面端不用退出。",
    importDesktop: "导入桌面端登录",
    importing: "正在读取桌面端登录…",
    saving: "正在保存…",
    loginHint:
      "在浏览器中完成登录——会使用浏览器里已登录的 devin 账号（如已登录）。",
    loginName: "账号会按邮箱自动命名，之后可在卡片上改名。",
    startLogin: "开始登录",
    waiting: "等待登录…",
    finishing: "已收到回调——正在完成登录…",
    copy: "复制",
    open: "打开",
    terminalInstead: "或者在终端里运行",
    cancel: "取消",
    logoutHint: "要登录与浏览器中不同的 devin 账号？先在浏览器中登出：",
    openLogout: "在默认浏览器中登出",
    copyLogout: "复制登出地址",
    currentHint:
      "保存 devin CLI 当前登录的账号。账号会按邮箱自动命名。",
    saveCurrent: "保存当前登录",
    tokenHint:
      "生成登录链接，在任何浏览器打开——登录后页面会显示一个 code。把 code（或 devin-session-token$… 开头的串）粘贴到下面。",
    getLink: "生成登录链接",
    codeOrToken: "Code 或令牌",
    codeOrTokenPh: "code 或 devin-session-token$…",
    saveToken: "保存",
    savedAccounts: "已保存的账号",
    noProfiles: "还没有保存的账号。<br>点上方「+ 添加账号」。",
    noProfilesSignedIn: "还没有保存的账号——当前登录的账号未保存。",
    unknown: "未知账号",
    swUse: "切换",
    swPar: "并行运行",
    swHint: "双击切换",
    renHint: "双击改名",
    swRef: "刷新额度",
    swDel: "删除",
    switching: (n) => `正在切换到 ${n}…`,
    active: "使用中",
    daily: "当天",
    weekly: "本周",
    resetNow: "即将重置",
    resetInHMS: (c) => `${c} 后重置`,
    resetInDH: (d, h) => `${d} 天 ${h} 小时后重置`,
    resetOn: (d) => `${d} 重置`,
    used: "已用",
    overage: "超额余额",
    planEnds: "周期结束",
    savedAs: (n) => `已保存为 ${n}`,
    savedTok: (n) => `已保存 ${n}`,
    signedInAs: (n) => `已登录 ${n}`,
    refAll: "刷新全部",
    copyHint: "点击复制",
    copied: "已复制",
    switchedTo: (w) => `已切换到 ${w}`,
    swScopeCli: "仅切换 CLI",
    swScopeDesktop: "仅切换桌面端",
    swMore: "更多切换方式",
    swToastBoth: (n) => `已切换到 ${n}（CLI + 桌面端）`,
    swToastCli: (n) => `已切换到 ${n}（仅 CLI）`,
    swToastDesktop: (n) => `已把桌面端切换到 ${n}`,
    swToastNoDesktop: (n) => `已切换到 ${n}（CLI；未检测到桌面端）`,
    swToastDesktopFail: (n, e) =>
      `已切换到 ${n}（CLI）——桌面端同步失败：${e}`,
    swQuitDesktop: "Devin 桌面端正在运行——请先完全退出再切换",
    restartWarn: (w, ps) => `已切换到 ${w}。重启这些进程后生效：${ps}`,
    launched: (n) => `已用 ${n} 启动并行 devin 会话`,
    runThis: (c) => `在终端中运行：${c}`,
    linkCopied: "链接已复制——可在任何浏览器或隐身窗口打开",
    tokenReq: "需要令牌",
    delSure: "确认删除",
    delSureLive: "仅删快照·保持登录",
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
  document.querySelectorAll("[data-i18n-ph]").forEach((el) => {
    el.placeholder = t(el.dataset.i18nPh);
  });
  document.querySelectorAll("[data-i18n-tt]").forEach((el) => {
    el.title = t(el.dataset.i18nTt);
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
// Whether the CLI is signed in, and whether that sign-in already matches a
// saved account — controls the "Save current" tab's visibility. Defaults
// keep the tab until the first refresh() says otherwise.
let signedIn = true;
let activeSaved = false;

function toast(msg, isErr = false) {
  const t2 = $("toast");
  t2.textContent = msg;
  t2.className = "toast" + (isErr ? " err" : "");
  clearTimeout(t2._h);
  t2.onclick = null;
  // Errors stay until dismissed via ✕ — 4s is too short to read a failure.
  if (!isErr) {
    t2._h = setTimeout(() => t2.classList.add("hidden"), 4000);
  } else {
    const x = document.createElement("button");
    x.className = "tx";
    x.textContent = "✕";
    x.onclick = () => t2.classList.add("hidden");
    t2.appendChild(x);
  }
}

async function refresh(force = false) {
  try {
    const [status, profiles] = await Promise.all([
      invoke("get_status"),
      invoke("list_profiles"),
    ]);
    renderStatus(status);
    renderProfiles(profiles, status, force);
  } catch (e) {
    toast(String(e), true);
  }
}

// Instant click feedback: el dims and stops taking input while fn runs.
// .busy also shields the card from the 5s poll's re-render.
async function pend(el, fn) {
  if (el?.classList.contains("busy")) return;
  el?.classList.add("busy");
  try {
    return await fn();
  } finally {
    el?.classList.remove("busy");
  }
}

function renderStatus(s) {
  const dsk = s.desktop && s.desktop.logged_in ? s.desktop : null;
  // Same account on both sides → merge into one row tagged CLI+DESKTOP;
  // a duplicated "same account" row is noise.
  const same =
    dsk &&
    s.auth?.logged_in &&
    ((s.auth.user_id && s.auth.user_id === dsk.user_id) ||
      (dsk.email && s.auth.email === dsk.email));

  const d = $("dotCli");
  $("tagCli").textContent = same ? t("bothTag") : t("cliTag");
  if (s.auth && s.auth.logged_in) {
    d.classList.add("on");
    $("labelCli").textContent = s.auth.email || s.auth.name || t("signedIn");
    $("labelCli").dataset.copy = s.auth.email || "";
    $("subCli").textContent = [
      s.auth.name && s.auth.name !== s.auth.email ? s.auth.name : null,
      s.auth.plan || s.auth.tier,
    ]
      .filter(Boolean)
      .join(" · ");
  } else {
    d.classList.remove("on");
    $("labelCli").textContent = t("notSignedIn");
    $("subCli").textContent = s.devin_installed ? "" : t("cliNotFound");
  }
  $("quotaCli").innerHTML = quotaHtml(s.usage || dsk?.usage);

  // Desktop row only exists when Desktop is installed AND signed in as a
  // different account — otherwise the merged row covers it.
  const row = $("rowDesktop");
  if (!s.desktop || same) {
    row.classList.add("hidden");
    return;
  }
  row.classList.remove("hidden");
  const dd = $("dotDesktop");
  if (s.desktop.logged_in) {
    dd.classList.add("on");
    $("labelDesktop").textContent =
      s.desktop.email || s.desktop.name || s.desktop.label || t("signedIn");
    $("labelDesktop").dataset.copy = s.desktop.email || "";
    const plan = s.desktop.usage?.plan || s.desktop.usage?.tier;
    $("subDesktop").textContent = [
      (s.desktop.name || s.desktop.label) !== s.desktop.email
        ? s.desktop.name || s.desktop.label
        : null,
      plan,
    ]
      .filter(Boolean)
      .join(" · ");
  } else {
    dd.classList.remove("on");
    $("labelDesktop").textContent = t("notSignedIn");
    $("subDesktop").textContent = "";
  }
  $("quotaDesktop").innerHTML = quotaHtml(s.desktop.usage);
}

// Usage → labeled progress bars. Fill = % remaining for daily/weekly,
// % remaining for ACU. Green → amber → red as it drains.
function quotaHtml(u) {
  if (!u) return "";
  const bar = (pct, cls) => {
    const p = Math.max(0, Math.min(100, pct));
    const c = cls || (p > 50 ? "ok" : p > 20 ? "warn" : "low");
    return `<div class="qbar"><i class="${c}" style="width:${p}%"></i></div>`;
  };
  // Carries the unix stamp so the 1s ticker can keep it live — survives
  // the 5s re-render since each tick re-queries the DOM.
  const rst = (unix) =>
    unix
      ? ` · <span class="reset" data-reset="${unix}">${resetText(unix)}</span>`
      : "";
  const row = (label, pct, val, cls) =>
    `<div class="qrow"><span class="qlab">${label}</span>` +
    `${bar(pct, cls)}<span class="qval">${val}</span></div>`;
  const rows = [];
  // Daily without a weekly row means the weekly window is exhausted —
  // show it as 0% rather than omitting it. A spent weekly also makes the
  // daily remainder unusable, so the daily bar stays muted, not green.
  const weekly = u.weekly_left ?? (u.daily_left != null ? 0 : null);
  const weeklyDead = weekly === 0;
  if (u.daily_left != null) {
    rows.push(
      row(
        t("daily"),
        u.daily_left,
        `${Math.round(u.daily_left)}%` + rst(u.daily_reset_unix),
        weeklyDead ? "mut" : undefined
      )
    );
  }
  if (weekly != null) {
    rows.push(
      row(
        t("weekly"),
        weekly,
        `${Math.round(weekly)}%` + rst(u.weekly_reset_unix)
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
  if (u.overage_micros != null && u.overage_micros !== 0) {
    const v = u.overage_micros / 1e6;
    foot.push(
      `${t("overage")} ${v >= 0 ? "+" : "-"}$${Math.abs(v).toFixed(2)}`
    );
  }
  if (u.plan_end) foot.push(`${t("planEnds")} ${esc(u.plan_end.slice(0, 10))}`);
  if (foot.length) rows.push(`<div class="qfoot">${foot.join(" · ")}</div>`);
  return rows.join("");
}

const r2 = (f) => Math.round(f * 100) / 100;

// Relative reset — <24h is a live HH:MM:SS countdown (a bare date is
// useless for a daily window), <8d is "Nd Nh", farther out a date.
const pad2 = (n) => String(n).padStart(2, "0");
function resetText(unix) {
  const s = Math.floor(unix - Date.now() / 1000);
  if (s <= 0) return t("resetNow");
  if (s < 86400) {
    const c = `${pad2(Math.floor(s / 3600))}:${pad2(
      Math.floor((s % 3600) / 60)
    )}:${pad2(s % 60)}`;
    return t("resetInHMS", c);
  }
  let d = Math.floor(s / 86400);
  let h = Math.round((s % 86400) / 3600);
  if (h === 24) {
    d += 1;
    h = 0;
  }
  if (d < 8) return t("resetInDH", d, h);
  const dt = new Date(unix * 1000);
  return t("resetOn", `${dt.getMonth() + 1}/${dt.getDate()}`);
}

// Sort score = how much usable quota remains. Weekly is the binding
// constraint so it dominates; daily breaks ties. ACU plans fall back to
// their remaining %. No data → -1 → sinks to the bottom.
function quotaScore(u) {
  if (!u) return -1;
  let w = u.weekly_left ?? u.daily_left;
  if (w == null && u.acu_limit > 0) {
    w = ((u.acu_limit - (u.acu_used || 0)) / u.acu_limit) * 100;
  }
  if (w == null) return -1;
  return w * 10 + (u.daily_left ?? w) * 0.1;
}

function renderProfiles(profiles, status, force = false) {
  const el = $("profiles");
  const active = profiles.find((p) => p.is_active);
  signedIn = !!status?.auth?.logged_in;
  activeSaved = !!active;
  const dsk = status?.desktop?.logged_in ? status.desktop : null;
  // Don't clobber an in-progress rename, armed delete, or busy card on
  // the 5s poll — an action's own refresh passes force to bypass this.
  if (!force && el.querySelector(".rename-in, .danger.armed, .busy, .menu"))
    return;
  if (!profiles.length) {
    el.innerHTML = `<p class="sub empty">${t(
      signedIn ? "noProfilesSignedIn" : "noProfiles"
    )}</p>`;
    if (signedIn) {
      const b = document.createElement("button");
      b.className = "primary";
      b.style.alignSelf = "center";
      b.textContent = t("saveCurrent");
      b.onclick = (e) => pend(e.currentTarget, saveCurrent);
      el.appendChild(b);
    }
    return;
  }
  el.innerHTML = "";
  const sorted = [...profiles].sort(
    (a, b) => quotaScore(b.usage) - quotaScore(a.usage)
  );
  for (const [i, p] of sorted.entries()) {
    // A profile can be live on CLI, on Desktop, on both (fully switched),
    // or neither. Match Desktop by user_id, falling back to email for
    // accounts saved before we recorded user_id.
    const deskActive =
      dsk &&
      ((p.user_id && p.user_id === dsk.user_id) ||
        (dsk.email && p.email === dsk.email));
    const bothActive = p.is_active && deskActive;
    const card = document.createElement("div");
    card.className = "profile" + (p.is_active ? " active" : "");
    if (!bothActive) {
      card.classList.add("switchable");
      card.title = t("swHint");
    }
    card.innerHTML = `
      <div class="top">
        <span class="idx">${i + 1}</span>
        <span class="name copyable" title="${esc(t("copyHint"))}">${esc(
          p.email || p.display_name || p.name
        )}</span>
        ${p.plan ? `<span class="sub">${esc(p.plan)}</span>` : ""}
        <span class="pname" title="${esc(t("renHint"))}">@${esc(p.name)}</span>
        ${p.is_active ? `<span class="badge">${t("cliTag")}</span>` : ""}
        ${deskActive ? `<span class="badge desk">${t("desktopTag")}</span>` : ""}
        <button class="icobtn danger" data-act="del" title="${t("swDel")}">✕</button>
      </div>
      <div class="quota-wrap">
        <div class="quota">${p.usage ? quotaHtml(p.usage) : ""}</div>
        <button class="icobtn" data-act="ref" title="${t("swRef")}">↻</button>
      </div>
      ${p.note ? `<div class="note">${esc(p.note)}</div>` : ""}
      ${
        bothActive
          ? ""
          : `<div class="actions"><span class="splitbtn"><button class="use" data-act="use">${t("swUse")}</button><button class="use caret" data-act="usemenu" title="${t("swMore")}">▾</button></span><button data-act="par">${t("swPar")}</button></div>`
      }`;
    const useBtn = card.querySelector('[data-act="use"]');
    if (useBtn)
      useBtn.onclick = () => pend(card, () => useProfile(p.name, "all"));
    const menuBtn = card.querySelector('[data-act="usemenu"]');
    if (menuBtn)
      menuBtn.onclick = (e) => {
        e.stopPropagation();
        const wrap = menuBtn.parentElement;
        const open = wrap.querySelector(".menu");
        document.querySelectorAll(".menu").forEach((m) => {
          if (m !== open) m.remove();
        });
        if (open) {
          open.remove();
          return;
        }
        const menu = document.createElement("div");
        menu.className = "menu";
        menu.innerHTML = `<button data-scope="cli">${t("swScopeCli")}</button><button data-scope="desktop">${t("swScopeDesktop")}</button>`;
        menu.onclick = (ev) => {
          const scope = ev.target.dataset.scope;
          if (!scope) return;
          menu.remove();
          pend(card, () => useProfile(p.name, scope));
        };
        wrap.appendChild(menu);
      };
    const parBtn = card.querySelector('[data-act="par"]');
    if (parBtn) parBtn.onclick = () => pend(parBtn, () => parallel(p.name));
    card.querySelector('[data-act="ref"]').onclick = (e) =>
      pend(e.currentTarget, () => refreshUsage(p.name));
    card.querySelector('[data-act="del"]').onclick = (e) =>
      armDelete(e.currentTarget, p.name, p.is_active);
    card.querySelector(".name").onclick = (e) => {
      e.stopPropagation();
      copyText(p.email || p.display_name || p.name);
    };
    card.querySelector(".pname").ondblclick = (e) => {
      e.stopPropagation();
      inlineRename(e.currentTarget, p.name);
    };
    if (!bothActive) {
      card.ondblclick = (e) => {
        if (e.target.closest("button, input, .menu")) return;
        pend(card, () => useProfile(p.name, "all"));
      };
    }
    el.appendChild(card);
  }
}

async function useProfile(name, scope) {
  // Immediate feedback — the backend still does a quota fetch, so the
  // switch itself isn't instant even without the devin CLI spawn.
  scope = scope || "all";
  toast(t("switching", name));
  try {
    const r = await invoke("use_profile", { name, scope });
    const who = name;
    const warn = $("warnBanner");
    if (r.restart_needed && r.restart_needed.length) {
      warn.innerHTML = "";
      const span = document.createElement("span");
      span.textContent = t("restartWarn", who, r.restart_needed.join(", "));
      const x = document.createElement("button");
      x.className = "tx";
      x.textContent = "✕";
      x.onclick = () => warn.classList.add("hidden");
      warn.append(span, x);
      warn.classList.remove("hidden");
      clearTimeout(warn._h);
      warn._h = setTimeout(() => warn.classList.add("hidden"), 15000);
    } else {
      warn.classList.add("hidden");
    }
    if (scope === "cli") toast(t("swToastCli", who));
    else if (scope === "desktop") toast(t("swToastDesktop", who));
    else if (r.desktop === "switched") toast(t("swToastBoth", who));
    else if (r.desktop === "unavailable") toast(t("swToastNoDesktop", who));
    else if ((r.desktop || "").startsWith("failed"))
      toast(t("swToastDesktopFail", who, r.desktop.slice(8)), true);
    else toast(t("switchedTo", who));
    refresh(true);
  } catch (e) {
    const s = String(e);
    toast(s === "devin_desktop_running" ? t("swQuitDesktop") : s, true);
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

// Copy on click — WebView2's navigator.clipboard exists but can reject
// without clipboard permission, so keep the execCommand fallback.
function copyText(s) {
  const legacy = () => {
    const ta = document.createElement("textarea");
    ta.value = s;
    ta.style.position = "fixed";
    ta.style.opacity = "0";
    document.body.append(ta);
    ta.select();
    document.execCommand("copy");
    ta.remove();
  };
  if (navigator.clipboard?.writeText) {
    navigator.clipboard.writeText(s).then(
      () => toast(t("copied")),
      () => {
        legacy();
        toast(t("copied"));
      }
    );
  } else {
    legacy();
    toast(t("copied"));
  }
}

// Double-click a name → swap it for an input. Enter/blur commits,
// Esc cancels. `done` guards against blur firing after Enter's commit.
function inlineRename(nameEl, old) {
  const input = document.createElement("input");
  input.className = "rename-in";
  input.value = old;
  nameEl.replaceWith(input);
  input.focus();
  input.select();
  let done = false;
  const finish = async (save) => {
    if (done) return;
    done = true;
    const to = input.value.trim();
    if (save && to && to !== old) {
      try {
        await invoke("rename_profile", { from: old, to });
      } catch (e) {
        toast(String(e), true);
      }
    }
    refresh(true);
  };
  input.onkeydown = (e) => {
    if (e.key === "Enter") finish(true);
    if (e.key === "Escape") finish(false);
  };
  input.onblur = () => finish(true);
}

async function refreshUsage(name) {
  try {
    await invoke("refresh_usage", { name });
    refresh(true);
  } catch (e) {
    toast(String(e), true);
  }
}

// Two-step delete — WebView2's native confirm() shows "tauri.localhost"
// as the origin, so the button itself arms for 3s instead.
function armDelete(btn, name, isActive) {
  if (btn.dataset.armed) {
    pend(btn.closest(".profile"), () => del(name));
    return;
  }
  btn.dataset.armed = "1";
  const label = btn.textContent;
  btn.textContent = t(isActive ? "delSureLive" : "delSure");
  btn.classList.add("armed");
  setTimeout(() => {
    delete btn.dataset.armed;
    btn.classList.remove("armed");
    btn.textContent = label;
  }, 3000);
}

async function del(name) {
  try {
    await invoke("delete_profile", { name });
    refresh(true);
  } catch (e) {
    toast(String(e), true);
  }
}

// devin.ai hides its logout link; offer it here so a second account can
// actually be signed into in the same browser profile.
const LOGOUT_URL = "https://app.devin.ai/logout";

// ---- Add sheet ----

$("addBtn").onclick = () => {
  // "Save current" only makes sense when there's an unsaved sign-in.
  const cur = document.querySelector('.tab[data-tab="current"]');
  const show = signedIn && !activeSaved;
  cur.classList.toggle("hidden", !show);
  if (!show && cur.classList.contains("active")) {
    document.querySelector('.tab[data-tab="login"]').click();
  }
  $("sheet").classList.remove("hidden");
};
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
    // The sign-out row only matters for browser-based flows.
    document
      .querySelector(".logoutrow")
      .classList.toggle("hidden", t2.dataset.tab === "desktop");
  };
}

async function saveCurrent() {
  toast(t("saving"));
  try {
    const p = await invoke("save_current", { name: "", note: "" });
    toast(t("savedAs", p.name));
    closeSheet();
    refresh(true);
  } catch (e) {
    toast(String(e), true);
  }
}

$("saveCurrent").onclick = (e) => pend(e.currentTarget, saveCurrent);

for (const id of ["labelCli", "labelDesktop"]) {
  $(id).onclick = (e) => {
    const v = e.currentTarget.dataset.copy;
    if (v) copyText(v);
  };
}

$("refAll").onclick = (e) =>
  pend(e.currentTarget, async () => {
    await invoke("refresh_all");
    await refresh(true);
  });

$("importDesktop").onclick = (e) =>
  pend(e.currentTarget, async () => {
    toast(t("importing"));
    try {
      const p = await invoke("import_desktop", { name: "" });
      toast(t("savedAs", p.name));
      closeSheet();
      refresh(true);
    } catch (err) {
      toast(String(err), true);
    }
  });

$("tokenLink").onclick = (e) =>
  pend(e.currentTarget, async () => {
    try {
      $("tokenUrl").value = await invoke("manual_start");
      $("tokenLinkRow").classList.remove("hidden");
    } catch (err) {
      toast(String(err), true);
    }
  });

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

$("saveToken").onclick = (e) =>
  pend(e.currentTarget, async () => {
    const code = $("tokenValue").value.trim();
    if (!code) return toast(t("tokenReq"), true);
    toast(t("saving"));
    try {
      const p = await invoke("manual_finish", { name: "", code });
      toast(t("savedTok", p.email || p.name));
      closeSheet();
      refresh(true);
    } catch (err) {
      toast(String(err), true);
    }
  });

$("startLogin").onclick = (e) =>
  pend(e.currentTarget, async () => {
    try {
      const offer = await invoke("start_login");
      loginId = offer.id;
      $("loginUrl").value = offer.url;
      $("loginHint").textContent = offer.hint;
      $("startLogin").classList.add("hidden");
      $("loginWait").classList.remove("hidden");
      loginTimer = setInterval(pollLogin, 600);
    } catch (err) {
      toast(String(err), true);
    }
  });

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

$("openLogout").onclick = () => {
  open(LOGOUT_URL).catch(() => {});
};
$("copyLogout").onclick = async () => {
  try {
    await navigator.clipboard.writeText(LOGOUT_URL);
    toast(t("linkCopied"));
  } catch {
    toast(LOGOUT_URL);
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
      refresh(true);
    } else if (r.kind === "working") {
      document.querySelector("#loginWait .wait-text").textContent =
        t("finishing");
    }
    // "waiting" → keep polling
  } catch (e) {
    toast(String(e), true);
    stopLogin();
  }
}

// Click anywhere outside a card's scope menu closes it.
document.addEventListener("click", (e) => {
  if (!e.target.closest(".splitbtn"))
    document.querySelectorAll(".menu").forEach((m) => m.remove());
});

// Keep quota countdowns ticking — one pass over [data-reset] spans a
// second. Cheap; re-rendered nodes are picked up on the next tick.
setInterval(() => {
  document.querySelectorAll("[data-reset]").forEach((el) => {
    el.textContent = resetText(+el.dataset.reset);
  });
}, 1000);

applyI18n();
setInterval(refresh, 5000);
