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
    loading: "Loading…",
    notSignedIn: "Not signed in",
    cliNotFound: "devin CLI not found",
    signedIn: "Signed in",
    addAccount: "+ Add account",
    tierOther: "OTHER",
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
    upgToPro: "best Pro-upgrade window",
    upgToMax: (d) => `Max upgrade · ~${d}d diff`,
    upgLast: (d) => `Max upgrade, last-day · ~${d}d`,
    upgNow: "optimal now",
    upgAvail: "available now",
    upgIn: (d) => `in ${d}`,
    upgOver: "window passed",
    upgProTip: (w) =>
      `Subscribe 1–2 days before the Sunday reset (${w}) — the month then covers the most weekly quotas`,
    upgMaxTip: (w) =>
      `Upgrade within 1–2 days before the last reset in the cycle (${w}) — two weekly quotas for the smallest diff`,
    savedAs: (n) => `Saved as ${n}`,
    savedTok: (n) => `Saved ${n}`,
    signedInAs: (n) => `Signed in as ${n}`,
    refAll: "Refresh all",
    filterPh: "Filter…",
    noMatch: "No matching account",
    vtabAccounts: "ACCOUNTS",
    vtabOpps: "OPPORTUNITIES",
    oppsEmpty: "No upgrade windows — no Free or Pro account with an active cycle.",
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
    loading: "加载中…",
    notSignedIn: "未登录",
    cliNotFound: "找不到 devin CLI",
    signedIn: "已登录",
    addAccount: "+ 添加账号",
    tierOther: "其他",
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
    upgToPro: "升 Pro 最佳窗口",
    upgToMax: (d) => `升 Max · ~${d}天差价`,
    upgLast: (d) => `升 Max 末日窗口 · ~${d}天差价`,
    upgNow: "现在最优",
    upgAvail: "现在可升",
    upgIn: (d) => `${d} 后`,
    upgOver: "窗口已过",
    upgProTip: (w) =>
      `在周日重置（${w}）前 1–2 天订 Pro —— 一个月覆盖的周限额最多`,
    upgMaxTip: (w) =>
      `在周期内最后一次周日重置（${w}）前 1–2 天内升级 —— 差价最小、能拿两个周限额`,
    savedAs: (n) => `已保存为 ${n}`,
    savedTok: (n) => `已保存 ${n}`,
    signedInAs: (n) => `已登录 ${n}`,
    refAll: "刷新全部",
    filterPh: "过滤…",
    noMatch: "没有匹配的账号",
    vtabAccounts: "账号",
    vtabOpps: "升级良机",
    oppsEmpty: "没有升级窗口 —— 没有处于有效订阅周期内的 Free 或 Pro 账号。",
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
// Filter box state + last poll results — typing re-renders without a
// backend round-trip.
let filterQuery = "";
// Tier chips toggle which plan tiers are visible. Persisted as the
// hidden set; migrates the older freeFilter/hideFree settings.
let hiddenTiers = new Set(
  (localStorage.getItem("hiddenTiers") || "").split(",").filter(Boolean)
);
{
  const ff = localStorage.getItem("freeFilter");
  if (ff === "hide" || localStorage.getItem("hideFree") === "1")
    hiddenTiers.add("FREE");
  if (ff === "only") ["MAX", "PRO", "OTHER"].forEach((k) => hiddenTiers.add(k));
}
let lastProfiles = [];
let lastStatus = null;

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

// Backend sentinels → friendly text; everything else passes through.
const errText = (e) => {
  const s = String(e);
  return s === "devin_desktop_running" ? t("swQuitDesktop") : s;
};

// A tray-menu switch that failed (e.g. Desktop still running) — the
// backend raises the window first, so the toast lands somewhere visible.
window.__TAURI__.event.listen("tray-error", (e) =>
  toast(errText(e.payload), true)
);

// Set for the duration of a refresh — the 5s tick skips when one is
// already in flight (explicit refresh() calls run regardless).
let refreshing = false;

async function refresh(force = false) {
  refreshing = true;
  try {
    const [status, profiles] = await Promise.all([
      invoke("get_status"),
      invoke("list_profiles"),
    ]);
    lastProfiles = profiles;
    lastStatus = status;
    renderStatus(status);
    renderProfiles(profiles, status, force);
  } catch (e) {
    toast(String(e), true);
  } finally {
    refreshing = false;
    // First render dismisses the splash — data or error toast, either
    // way the loading screen has served its purpose.
    $("splash")?.classList.add("gone");
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
  // a duplicated "same account" row is noise. The backend decides.
  const same = !!(dsk && s.desktop.same_as_cli);

  const d = $("dotCli");
  d.classList.toggle("both", same);
  $("tagCli").textContent = same ? t("bothTag") : t("cliTag");
  if (s.auth && s.auth.logged_in) {
    d.classList.add("on");
    $("labelCli").textContent = s.auth.email || s.auth.name || t("signedIn");
    $("labelCli").dataset.copy = s.auth.email || "";
    $("subCli").innerHTML = [
      s.auth.name && s.auth.name !== s.auth.email ? esc(s.auth.name) : null,
      planTag(s.auth.plan || s.auth.tier, s.usage?.tier),
    ]
      .filter(Boolean)
      .join(" · ");
  } else {
    d.classList.remove("on");
    $("labelCli").textContent = t("notSignedIn");
    $("subCli").textContent = s.devin_installed ? "" : t("cliNotFound");
  }
  $("quotaCli").innerHTML = quotaHtml(s.usage || (same ? dsk.usage : null));

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
    $("subDesktop").innerHTML = [
      (s.desktop.name || s.desktop.label) !== s.desktop.email
        ? esc(s.desktop.name || s.desktop.label)
        : null,
      planTag(s.desktop.usage?.plan, s.desktop.usage?.tier),
    ]
      .filter(Boolean)
      .join(" · ");
    $("subDesktop").title = "";
    $("subDesktop").classList.remove("err");
  } else {
    dd.classList.remove("on");
    $("labelDesktop").textContent = t("notSignedIn");
    const sub = $("subDesktop");
    sub.textContent = s.desktop.error || "";
    sub.title = s.desktop.error || "";
    sub.classList.toggle("err", !!s.desktop.error);
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
  if (u.plan_end) {
    const ts = Date.parse(u.plan_end);
    if (isNaN(ts)) {
      foot.push(`${t("planEnds")} ${esc(u.plan_end.slice(0, 10))}`);
    } else {
      const d = new Date(ts);
      const when =
        `${String(d.getMonth() + 1).padStart(2, "0")}-` +
        `${String(d.getDate()).padStart(2, "0")} ` +
        `${String(d.getHours()).padStart(2, "0")}:` +
        `${String(d.getMinutes()).padStart(2, "0")}`;
      const unix = Math.floor(ts / 1000);
      const soon = ts - Date.now() < 7 * 864e5;
      foot.push(
        `<span class="${soon ? "soon" : ""}">${t("planEnds")} ` +
          `${when} · ` +
          `<span data-planend="${unix}">${planLeft(unix)}</span></span>`
      );
    }
  }
  if (foot.length) rows.push(`<div class="qfoot">${foot.join(" · ")}</div>`);
  return rows.join("");
}

// ---- upgrade-timing hints ----
// Weekly quota resets Sun 16:00 Beijing (08:00 UTC), fixed for every
// account — u.weekly_reset_unix carries it, else computed. Free→Pro is
// best 1–2 days before a reset (one month then covers the most weekly
// windows). Pro→Max works before ANY reset — the upgrade resets quota
// immediately, the Sunday reset refills it again — but the diff is
// prorated on remaining cycle days, so the LAST reset in the cycle is
// the sweet spot ("now-optimal"); earlier ones show as merely
// "available". No reset left inside the cycle → last-day upgrade still
// swaps ~1d diff for a fresh quota.
const WEEK = 604800;
function nextWeeklyReset(now, seed) {
  if (seed) return seed;
  const d = new Date(now * 1000);
  let t = Math.floor(
    Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate(), 8) / 1000
  );
  while (t <= now || new Date(t * 1000).getUTCDay() !== 0) t += 86400;
  return t;
}

// The account's current upgrade window, or null: {k, w0, w1, pe, last}
// — k = pro / max / last(day), w0..w1 the window, pe the plan end,
// last = "1" when the Pro window targets the cycle's last reset.
function upgWindow(p, u) {
  const tier = tierOf(p.plan, u?.tier);
  const now = Date.now() / 1000;
  const R = nextWeeklyReset(Math.floor(now), u?.weekly_reset_unix);
  if (tier === "FREE") {
    return { k: "pro", w0: R - 2 * 86400, w1: R, pe: 0, last: "" };
  }
  if (tier !== "PRO" || !u?.plan_end) return null;
  const end = Date.parse(u.plan_end) / 1000;
  if (!(end > now)) return null;
  if (R < end) {
    // Next upcoming reset works too — flag whether it's the last one
    // inside the cycle, which is what makes it optimal.
    let rl = R;
    while (rl + WEEK < end) rl += WEEK;
    return { k: "max", w0: R - 2 * 86400, w1: R, pe: end, last: rl === R ? "1" : "" };
  }
  return { k: "last", w0: end - 2 * 86400, w1: end, pe: end, last: "" };
}

function upgradeHint(p, u) {
  const w = upgWindow(p, u);
  if (!w) return "";
  const d = new Date(w.w1 * 1000);
  const wt = `${pad2(d.getMonth() + 1)}-${pad2(d.getDate())} ${pad2(
    d.getHours()
  )}:${pad2(d.getMinutes())}`;
  const tip = w.k === "pro" ? t("upgProTip", wt) : t("upgMaxTip", wt);
  return (
    `<div class="upg" title="${esc(tip)}" data-k="${w.k}" data-last="${w.last}" ` +
    `data-w0="${Math.floor(w.w0)}" data-w1="${Math.floor(w.w1)}" data-pe="${Math.floor(
      w.pe
    )}"><div class="heat"><i></i></div><span class="upgtxt"></span></div>`
  );
}

// Per-tick fill: the heat bar warms up over the ~5 days before the
// window. Full heat (green) only for the optimal window — the cycle's
// last reset; earlier resets are valid but cap at warm amber.
function updateUpg(el) {
  const now = Date.now() / 1000;
  const w0 = +el.dataset.w0;
  const w1 = +el.dataset.w1;
  const pe = +el.dataset.pe;
  const k = el.dataset.k;
  const optimal = k !== "max" || el.dataset.last === "1";
  const inWin = now >= w0 && now <= w1;
  const heat = inWin
    ? optimal ? 1 : 0.7
    : Math.max(0, Math.min(1, 1 - (w0 - now) / (5 * 86400))) * (optimal ? 1 : 0.7);
  const bar = el.querySelector(".heat i");
  bar.style.width = `${Math.round(heat * 100)}%`;
  bar.className = inWin ? (optimal ? "hot" : "warm") : heat > 0.35 ? "warm" : "";
  const cost = Math.max(1, Math.ceil((pe - now) / 86400));
  const what =
    k === "pro"
      ? t("upgToPro")
      : k === "max"
        ? t("upgToMax", cost)
        : t("upgLast", cost);
  const status = inWin
    ? optimal ? t("upgNow") : t("upgAvail")
    : now > w1 ? t("upgOver") : t("upgIn", planLeft(w0));
  el.querySelector(".upgtxt").textContent = `${what} · ${status}`;
}

// Precise plan-end countdown — the API stamps a full RFC3339 instant,
// not just a date, so granularity tightens as expiry nears: ≥7d shows
// days, <7d adds hours, <1d is h+m, <1h is minutes. The same 1s ticker
// keeps it fresh and flips the parent to warn under 7 days.
function planLeft(unix) {
  const s = Math.max(0, Math.ceil(unix - Date.now() / 1000));
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  if (d >= 7) return `${d}d`;
  if (d >= 1) return `${d}d ${h}h`;
  if (s >= 3600) return `${Math.floor(s / 3600)}h ${m}m`;
  return `${m}m`;
}

const r2 = (f) => Math.round(f * 100) / 100;

// The one tier rule — plan chips (class) and filter chips (key) share
// it. `tier` is the server's teamsTier (authoritative — planName lags a
// mid-cycle upgrade); regex on the plan text is the fallback.
function tierOf(plan, tier) {
  if (["MAX", "PRO", "FREE"].includes(tier)) return tier;
  return /max|ultra/i.test(plan || "")
    ? "MAX"
    : /pro|team|business/i.test(plan || "")
      ? "PRO"
      : /free/i.test(plan || "")
        ? "FREE"
        : "OTHER";
}

// Plan tier chip: free stays muted, pro = accent, max = gold. Shows the
// authoritative tier label when known (planName can be stale).
function planTag(plan, tier) {
  const t2 = tierOf(plan, tier);
  const cls = { MAX: "max", PRO: "pro" }[t2] || "";
  const label = tier || plan;
  if (!label) return "";
  return `<span class="plantag ${cls}">${esc(label)}</span>`;
}

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

// Which usage a card shows: the live banner object when this account is
// signed in somewhere, else its stored snapshot.
function liveUsageOf(p, status) {
  if (p.is_active) return status?.usage ?? p.usage;
  if (p.desktop_active) return status?.desktop?.usage ?? p.usage;
  return p.usage;
}

// Centralized upgrade windows live on their own tab — every window not
// yet past is listed, soonest first, so the tab is never empty while an
// account could still upgrade. Actionable = open now or opening within
// ~1 day; only those count toward the badge and light the tab title,
// and future rows render dimmed until their turn. Click a row to jump
// back to that account's card.
function renderOpps(profiles, status) {
  const list = $("oppsList");
  const now = Date.now() / 1000;
  const rows = profiles
    .map((p) => ({ p, u: liveUsageOf(p, status), w: upgWindow(p, liveUsageOf(p, status)) }))
    .filter((r) => r.w && now <= r.w.w1)
    .sort((a, b) => a.w.w0 - b.w.w0);
  const actionable = rows.filter((r) => now >= r.w.w0 - 86400).length;
  const badge = $("oppBadge");
  badge.textContent = actionable;
  badge.classList.toggle("hidden", !actionable);
  $("tabOpps").classList.toggle("hot", actionable > 0);
  $("oppsEmpty").classList.toggle("hidden", rows.length > 0);
  list.innerHTML = rows
    .map(
      ({ p, u, w }) =>
        `<div class="opprow${now < w.w0 - 86400 ? " future" : ""}" data-acct="${esc(
          p.name
        )}">` +
        `<span class="oppname">${esc(p.email || p.display_name || p.name)}</span>` +
        planTag(p.plan, u?.tier) +
        upgradeHint(p, u) +
        `</div>`
    )
    .join("");
  list.querySelectorAll(".upg").forEach(updateUpg);
}

function renderProfiles(profiles, status, force = false) {
  const el = $("profiles");
  const active = profiles.find((p) => p.is_active);
  signedIn = !!status?.auth?.logged_in;
  activeSaved = !!active;
  renderOpps(profiles, status);
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
  // Tier census of ALL saved accounts — each present tier is a toggle
  // chip: lit = shown, dimmed = filtered out.
  const counts = {};
  profiles.forEach((p) => {
    const k = tierOf(p.plan, p.usage?.tier);
    counts[k] = (counts[k] || 0) + 1;
  });
  const tc = $("tierCounts");
  tc.innerHTML = profiles.length ? `${profiles.length} ` : "";
  ["MAX", "PRO", "FREE", "OTHER"]
    .filter((k) => counts[k])
    .forEach((k) => {
      const c = document.createElement("span");
      c.className =
        `tierchip ${k.toLowerCase()}` + (hiddenTiers.has(k) ? "" : " on");
      c.textContent = `${k === "OTHER" ? t("tierOther") : k} ${counts[k]}`;
      c.onclick = (e) => {
        e.stopPropagation();
        if (hiddenTiers.has(k)) hiddenTiers.delete(k);
        else hiddenTiers.add(k);
        localStorage.setItem("hiddenTiers", [...hiddenTiers].join(","));
        renderProfiles(lastProfiles, lastStatus, true);
      };
      tc.appendChild(c);
    });
  const q = filterQuery.trim().toLowerCase();
  const shown = profiles.filter(
    (p) =>
      !hiddenTiers.has(tierOf(p.plan, p.usage?.tier)) &&
      (!q ||
        [p.email, p.name, p.display_name].some(
          (f) => f && f.toLowerCase().includes(q)
        ))
  );
  // A card for the live account renders the SAME usage object the banner
  // shows — the banner refreshes every ~60s while meta snapshots can be
  // 5min stale, so without this the same account could show two numbers.
  const usageOf = (p) => liveUsageOf(p, status);
  const sorted = [...shown].sort(
    (a, b) => quotaScore(usageOf(b)) - quotaScore(usageOf(a))
  );
  if (!sorted.length) {
    el.innerHTML = `<p class="sub empty">${t("noMatch")}</p>`;
    return;
  }
  for (const [i, p] of sorted.entries()) {
    // A profile can be live on CLI, on Desktop, on both (fully switched),
    // or neither. Match Desktop by user_id, falling back to email for
    // accounts saved before we recorded user_id.
    const deskActive = p.desktop_active;
    const bothActive = p.is_active && deskActive;
    // Switching a side that's already on this account is a no-op — the
    // button targets only the side(s) that aren't live yet.
    const mainScope = !p.is_active
      ? deskActive
        ? "cli"
        : "all"
      : "desktop";
    const card = document.createElement("div");
    card.dataset.acct = p.name;
    card.className =
      "profile" +
      (bothActive
        ? " active-both"
        : p.is_active
          ? " active"
          : deskActive
            ? " active-desk"
            : "");
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
        ${planTag(p.plan, p.usage?.tier)}
        <span class="pname" title="${esc(t("renHint"))}">@${esc(p.name)}</span>
        ${p.is_active ? `<span class="badge">${t("cliTag")}</span>` : ""}
        ${deskActive ? `<span class="badge desk">${t("desktopTag")}</span>` : ""}
        <button class="icobtn danger" data-act="del" title="${t("swDel")}">✕</button>
      </div>
      <div class="quota-wrap">
        <div class="quota">${quotaHtml(usageOf(p))}</div>
        <button class="icobtn" data-act="ref" title="${t("swRef")}">↻</button>
      </div>
      ${upgradeHint(p, usageOf(p))}
      ${p.note ? `<div class="note">${esc(p.note)}</div>` : ""}
      ${
        bothActive
          ? ""
          : `<div class="actions"><span class="splitbtn"><button class="use" data-act="use">${t(
              mainScope === "all"
                ? "swUse"
                : mainScope === "cli"
                  ? "swScopeCli"
                  : "swScopeDesktop",
            )}</button>${
              mainScope === "all"
                ? `<button class="use caret" data-act="usemenu" title="${t("swMore")}">▾</button>`
                : ""
            }</span><button data-act="par">${t("swPar")}</button></div>`
      }`;
    // Fill the upgrade hint NOW — the ticker only fires once a second,
    // and cards rebuild every poll, so without this the bar flashes
    // 0→full and the text sits empty on every re-render.
    const upg = card.querySelector(".upg");
    if (upg) updateUpg(upg);
    const useBtn = card.querySelector('[data-act="use"]');
    if (useBtn)
      useBtn.onclick = () => pend(card, () => useProfile(p.name, mainScope));
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
        pend(card, () => useProfile(p.name, mainScope));
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
    toast(errText(e), true);
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
function copyText(s, msg) {
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
  const done = () => toast(msg || t("copied"));
  if (navigator.clipboard?.writeText) {
    navigator.clipboard.writeText(s).then(done, () => {
      legacy();
      done();
    });
  } else {
    legacy();
    done();
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

// Main view tabs — accounts list vs the opportunities rollup.
function setView(v) {
  localStorage.setItem("ui_tab", v);
  const opps = v === "opps";
  $("tabAccounts").classList.toggle("active", !opps);
  $("tabOpps").classList.toggle("active", opps);
  $("viewAccounts").classList.toggle("hidden", opps);
  $("viewOpps").classList.toggle("hidden", !opps);
  document
    .querySelectorAll(".acct-only")
    .forEach((e) => e.classList.toggle("hidden", opps));
}
$("tabAccounts").onclick = () => setView("accounts");
$("tabOpps").onclick = () => setView("opps");

// Jump from an upgrade-window row to that account's card.
$("oppsList").onclick = (e) => {
  const row = e.target.closest("[data-acct]");
  if (!row) return;
  setView("accounts");
  document
    .querySelector(`.profile[data-acct="${CSS.escape(row.dataset.acct)}"]`)
    ?.scrollIntoView({ behavior: "smooth", block: "center" });
};

$("filterIn").oninput = (e) => {
  filterQuery = e.currentTarget.value;
  if (lastStatus) renderProfiles(lastProfiles, lastStatus, true);
};

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

$("copyTokenUrl").onclick = () =>
  copyText($("tokenUrl").value, t("linkCopied"));

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
      loginTimer = setTimeout(pollLogin, 600);
    } catch (err) {
      toast(String(err), true);
    }
  });

$("openUrl").onclick = () => {
  const url = $("loginUrl").value;
  if (url) open(url).catch(() => {});
};

$("copyUrl").onclick = () =>
  copyText($("loginUrl").value, t("linkCopied"));

$("openLogout").onclick = () => {
  open(LOGOUT_URL).catch(() => {});
};
$("copyLogout").onclick = () => copyText(LOGOUT_URL, t("linkCopied"));

$("cancelLogin").onclick = () => {
  if (loginId != null) invoke("cancel_login", { id: loginId }).catch(() => {});
  stopLogin();
};

function stopLogin() {
  loginId = null;
  clearTimeout(loginTimer);
  $("startLogin").classList.remove("hidden");
  $("loginWait").classList.add("hidden");
}

// Self-scheduling, not setInterval — a new poll must never fire while the
// previous one is still awaiting (GotCreds removes the session server-side,
// so an overlapping poll sees "no such login session").
async function pollLogin() {
  if (loginId == null) return;
  const id = loginId;
  try {
    const r = await invoke("poll_login", { id, name: "", note: "" });
    if (r.kind === "done") {
      toast(t("signedInAs", r.profile.email || r.profile.name));
      stopLogin();
      closeSheet();
      refresh(true);
      return;
    }
    if (r.kind === "working") {
      document.querySelector("#loginWait .wait-text").textContent =
        t("finishing");
    }
    // "waiting"/"working" → next poll, only if this round is still live
    if (loginId === id) loginTimer = setTimeout(pollLogin, 600);
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
  document.querySelectorAll(".upg").forEach(updateUpg);
  document.querySelectorAll("[data-planend]").forEach((el) => {
    const unix = +el.dataset.planend;
    el.textContent = planLeft(unix);
    el.parentElement.classList.toggle(
      "soon",
      unix * 1000 - Date.now() < 7 * 864e5
    );
  });
}, 1000);

applyI18n();
setView(localStorage.getItem("ui_tab") || "accounts");
setInterval(() => {
  if (!refreshing) refresh();
}, 5000);
