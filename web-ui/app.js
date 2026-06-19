const storageKeys = {
  theme: "eyeforge-web-theme",
  language: "eyeforge-web-language",
  groups: "eyeforge-ai-groups",
  activeGroup: "eyeforge-active-ai-group",
};

const MAX_LOG_ENTRIES = 80;
const MAX_GROUP_MESSAGES = 80;

const state = {
  ws: null,
  connected: false,
  authenticated: false,
  theme: localStorage.getItem(storageKeys.theme) || "dark",
  language: localStorage.getItem(storageKeys.language) || "zh",
  groups: loadGroups(),
  activeGroupId: localStorage.getItem(storageKeys.activeGroup) || "",
  logs: [],
  channelMode: "list",
  channelKind: "gateway",
  channelConfig: {},
  channelsLoaded: false,
  voiceLoaded: false,
  wechatQrKey: "",
  wechatQrImage: "",
  wechatQrStatus: "",
  wechatQrTimer: null,
  wechatQrPolling: false,
  aiGroupMode: "chat",
  aiGroupEditIndex: null,
  aiGroupRunning: false,
};

const dict = {
  en: {
    sidebar_subtitle: "Rust Gateway Console",
    sidebar_gateway: "Gateway :9178",
    nav_dashboard: "Dashboard",
    nav_ai_groups: "AI Groups",
    nav_settings: "Settings",
    nav_channels: "Channels",
    nav_voice: "Voice",
    nav_logs: "Logs",
    hero_eyebrow: "Rust Native Gateway",
    hero_title: "EyeForge browser console",
    hero_body:
      "The Rust gateway serves this UI and exposes WebSocket execution, channel status, voice transcription, and AI group collaboration.",
    theme_button: "Theme",
    metric_gateway_label: "Gateway",
    metric_socket_label: "WebSocket",
    metric_socket_detail: "Task execution and structured actions share the same Rust runtime.",
    metric_mode_label: "Mode",
    metric_mode_detail: "Visual hints only affect the UI. Real execution follows backend actions.",
    task_title: "Task Composer",
    task_vision_hint: "Vision hint",
    task_placeholder:
      'Examples:\nNatural language: Open Notepad and type a short note\nStructured JSON: {"actions":[{"type":"wait","seconds":1},{"type":"complete","result":"ok"}]}',
    connect_button: "Connect",
    disconnect_button: "Disconnect",
    run_button: "Run Task",
    token_label: "Token",
    token_placeholder: "Optional token",
    result_title: "Latest Result",
    result_empty_title: "No result yet",
    result_empty_body: "Run a Rust gateway task or use voice transcription below.",
    screenshot_empty: "Screenshot output will appear here when a task returns image data.",
    settings_title: "Settings",
    settings_local: "Local",
    settings_webui_toggle: "Enable Web UI gateway",
    settings_vision_toggle: "Enable vision hint",
    settings_skill_toggle: "Enable Skill system",
    settings_hint:
      "These browser switches affect the Web UI immediately. Persistent backend settings are saved from the desktop app.",
    channels_title: "Channel Matrix",
    channels_subtitle: "Open a channel list first. Use the floating plus button to create or configure a channel.",
    create_channel: "Create Channel",
    channel_type: "Channel Type",
    back: "Back",
    save_channel: "Save Channel",
    channel_saved: "Channel saved",
    add_channel: "Add channel",
    refresh_button: "Refresh",
    wechat_qr_login: "QR Login",
    wechat_qr_hint: "Scan with WeChat. The Bot Token will be saved after confirmation.",
    wechat_qr_loading: "Requesting QR code...",
    wechat_qr_waiting: "Waiting for scan confirmation...",
    wechat_qr_success: "WeChat login succeeded",
    voice_title: "Voice Console",
    devices_button: "Devices",
    voice_devices_title: "Input Devices",
    voice_transcribe_title: "Native Transcription",
    voice_seconds_label: "Seconds",
    voice_record_button: "Record + Transcribe",
    voice_empty_title: "No transcription yet",
    voice_empty_body:
      "The Rust backend records from the default microphone and transcribes when a supported provider is configured.",
    logs_title: "Event Log",
    logs_empty_title: "No logs yet",
    logs_empty_body: "Events from gateway, AI groups, channels, and voice will appear here.",
    clear_button: "Clear",
    online: "Connected",
    pending: "Awaiting auth",
    offline: "Offline",
    waiting: "Waiting for the Rust gateway",
    connectedDetail: "WebSocket is open and auth has passed",
    awaitingDetail: "Socket is open and waiting for auth",
    vision: "Vision enabled",
    command: "Command only",
    boot: "EyeForge Web UI is ready",
    noResult: "No result yet",
    noVoice: "No transcription yet",
    channelsLoading: "Loading channels...",
    devicesLoading: "Loading input devices...",
    noChannels: "No channels",
    noDevices: "No input devices found",
    defaultDevice: "default",
    voiceRecordingTitle: "Recording...",
    voiceRecordingBody: (seconds) => `Capturing ${seconds} second(s) from the default microphone`,
    voiceErrorTitle: "Voice error",
    connectError: "WebSocket is not connected",
    taskEmpty: "Task is empty",
    socketError: "WebSocket error",
    alreadyConnected: "Already connected",
    openingSocket: (url) => `Opening ${url}`,
    socketClosed: "socket closed",
    manual: "manual",
    open: "Open",
    idle: "Idle",
    error: "Error",
    auth: "Auth",
    socket: "Socket",
    task: "Task",
    voice: "Voice",
    bootTitle: "Boot",
    ai_empty_title: "No AI group yet",
    ai_empty_body: "Create a group first. EyeForge will not create Dragon Group or any other default room.",
    ai_group_name_label: "Group name",
    ai_group_name_placeholder: "Project room",
    create_group: "Create Group",
    group_list_title: "Groups",
    group_settings: "Group Settings",
    group_desc: "Coordinate daily work across specialized agents",
    no_group_members: "No members yet",
    no_group_agents: "No AI agents yet",
    add_member: "Add Local Note",
    add_ai: "Add AI Assistant",
    edit_group: "Edit Group",
    people_title: "People",
    ai_title: "AI",
    member_name_prompt: "Member name",
    member_role_prompt: "Member role",
    ai_name_prompt: "AI name",
    ai_role_prompt: "AI role",
    ai_kind_prompt: "AI type: codex / openclaw / claude / opencode / astrbot / api",
    ai_endpoint_prompt: "AI endpoint",
    member_form_title: "Add Local Member",
    ai_form_title: "Add AI Assistant",
    save_member: "Save Member",
    save_ai: "Save AI Assistant",
    cancel: "Cancel",
    note_label: "Note",
    endpoint_label: "Endpoint",
    ws_endpoint_label: "WebSocket URL",
    hapi_endpoint_label: "HAPI Endpoint",
    api_endpoint_label: "API Endpoint",
  mixed_agent_hint: "OpenClaw/AstrBot use WebSocket. OpenCode/Codex can be local. Claude/HAPI use HTTP. api supports OpenAI-compatible endpoints.",
    kind_label: "Assistant Type",
    group_name_prompt: "Group name",
    group_created: "Group created",
    member_added: "Member added",
    ai_added: "AI added",
    group_updated: "Group updated",
    config_saved: "Synced to desktop config",
    config_load_failed: "Failed to load desktop config",
    config_save_failed: "Failed to save desktop config",
    local_member_note: "Local note only. It does not invite an external account.",
    ai_placeholder: "Mention a member or type a goal",
    system_member: "Group Assistant",
    system_message: "The group is ready. Add members or AI agents, then route tasks by role here.",
    collaborate_button: "Collaborate",
    collaboration_running: "Collaborating...",
    collaboration_empty: "Type a goal before starting collaboration.",
    collaboration_error: "Collaboration failed",
    user_member: "You",
  },
  zh: {
    sidebar_subtitle: "Rust 网关控制台",
    sidebar_gateway: "网关 :9178",
    nav_dashboard: "控制台",
    nav_ai_groups: "AI 群组",
    nav_settings: "设置",
    nav_channels: "通道",
    nav_voice: "语音",
    nav_logs: "日志",
    hero_eyebrow: "Rust 原生网关",
    hero_title: "EyeForge 浏览器控制台",
    hero_body: "Rust 网关托管此页面，统一提供 WebSocket 执行、通道状态、语音转写和 AI 群组协作入口。",
    theme_button: "主题",
    metric_gateway_label: "网关",
    metric_socket_label: "WebSocket",
    metric_socket_detail: "任务执行和结构化动作共用同一条 Rust 运行链。",
    metric_mode_label: "模式",
    metric_mode_detail: "界面提示只影响前端显示，真实执行以后端动作链为准。",
    task_title: "任务输入",
    task_vision_hint: "视觉提示",
    task_placeholder:
      '示例：\n自然语言：打开记事本并输入一段短句\n结构化 JSON：{"actions":[{"type":"wait","seconds":1},{"type":"complete","result":"ok"}]}',
    connect_button: "连接",
    disconnect_button: "断开",
    run_button: "执行任务",
    token_label: "令牌",
    token_placeholder: "可选令牌",
    result_title: "最新结果",
    result_empty_title: "还没有结果",
    result_empty_body: "运行一个 Rust 网关任务，或使用下方语音转写。",
    screenshot_empty: "当任务返回图像数据时，截图预览会显示在这里。",
    settings_title: "设置",
    settings_local: "本地",
    settings_webui_toggle: "启用 Web UI 网关",
    settings_vision_toggle: "启用视觉提示",
    settings_skill_toggle: "启用 Skill 系统",
    settings_hint: "这些浏览器开关会立即影响 Web UI；需要持久保存的后端设置请在桌面端保存。",
    channels_title: "通道矩阵",
    channels_subtitle: "进入后先显示通道列表。点击右下角加号创建或配置通道。",
    create_channel: "创建通道",
    channel_type: "通道类型",
    back: "返回",
    save_channel: "保存通道",
    channel_saved: "通道已保存",
    add_channel: "添加通道",
    refresh_button: "刷新",
    voice_title: "语音控制台",
    devices_button: "设备",
    voice_devices_title: "输入设备",
    voice_transcribe_title: "原生语音转写",
    voice_seconds_label: "录音秒数",
    voice_record_button: "录音并转写",
    voice_empty_title: "还没有转写结果",
    voice_empty_body: "Rust 后端会从默认麦克风录音，并在配置支持的提供商后执行转写。",
    logs_title: "事件日志",
    logs_empty_title: "还没有日志",
    logs_empty_body: "网关、AI 群组、通道和语音事件会显示在这里。",
    clear_button: "清空",
    online: "已连接",
    pending: "待认证",
    offline: "离线",
    waiting: "等待连接 Rust 网关",
    connectedDetail: "WebSocket 已连接，认证通过",
    awaitingDetail: "连接已建立，正在等待认证结果",
    vision: "视觉已开启",
    command: "仅命令",
    boot: "EyeForge Web UI 已就绪",
    noResult: "还没有结果",
    noVoice: "还没有转写结果",
    channelsLoading: "正在加载通道状态...",
    devicesLoading: "正在加载输入设备...",
    noChannels: "没有通道信息",
    noDevices: "没有发现输入设备",
    defaultDevice: "默认",
    voiceRecordingTitle: "正在录音...",
    voiceRecordingBody: (seconds) => `正在从默认麦克风录制 ${seconds} 秒`,
    voiceErrorTitle: "语音错误",
    connectError: "WebSocket 尚未连接",
    taskEmpty: "任务内容不能为空",
    socketError: "WebSocket 连接错误",
    alreadyConnected: "已经连接",
    openingSocket: (url) => `正在连接 ${url}`,
    socketClosed: "连接已关闭",
    manual: "手动",
    open: "已打开",
    idle: "空闲",
    error: "错误",
    auth: "认证",
    socket: "连接",
    task: "任务",
    voice: "语音",
    bootTitle: "启动",
    ai_empty_title: "还没有 AI 群组",
    ai_empty_body: "先创建一个自己的群聊。EyeForge 不会预置“龙虾群”或其他默认群。",
    ai_group_name_label: "群聊名称",
    ai_group_name_placeholder: "项目协作群",
    create_group: "创建群聊",
    group_list_title: "群聊",
    group_settings: "群聊设置",
    group_desc: "协助多个专长代理完成日常任务",
    no_group_members: "还没有成员",
    no_group_agents: "还没有 AI 成员",
    add_member: "添加本地成员备注",
    add_ai: "添加 AI 助手",
    edit_group: "编辑群信息",
    people_title: "成员",
    ai_title: "AI",
    member_name_prompt: "成员名称",
    member_role_prompt: "成员角色",
    ai_name_prompt: "AI 名称",
    ai_role_prompt: "AI 角色",
    ai_kind_prompt: "AI 类型：codex / openclaw / claude / opencode / astrbot",
    ai_endpoint_prompt: "AI 端点",
    group_name_prompt: "群聊名称",
    group_created: "群聊已创建",
    member_added: "成员已添加",
    ai_added: "AI 已添加",
    group_updated: "群聊已更新",
    config_saved: "已同步到桌面端配置",
    config_load_failed: "读取桌面端配置失败",
    config_save_failed: "保存桌面端配置失败",
    local_member_note: "仅作为本地成员备注，不会邀请外部账号入群。",
    ai_placeholder: "@成员 或输入任务目标",
    system_member: "群组助手",
    system_message: "群聊已创建。添加成员或 AI 后，可以在这里按角色分配任务。",
    collaborate_button: "协作",
    collaboration_running: "正在协作...",
    collaboration_empty: "请先输入任务目标。",
    collaboration_error: "协作失败",
    user_member: "你",
  },
};

Object.assign(dict.zh, {
  member_form_title: "添加本地成员",
  ai_form_title: "添加 AI 助手",
  save_member: "保存成员",
  save_ai: "保存 AI 助手",
  cancel: "取消",
  note_label: "备注",
  endpoint_label: "连接地址",
  ws_endpoint_label: "WebSocket 地址",
  hapi_endpoint_label: "HAPI 地址",
  mixed_agent_hint: "OpenClaw 和 AstrBot 使用 WebSocket；OpenCode、Codex、Claude Code 使用 HAPI。",
  wechat_qr_login: "扫码登录",
  wechat_qr_hint: "请使用微信扫码，确认后 Bot Token 会自动保存。",
  wechat_qr_loading: "正在获取二维码...",
  wechat_qr_waiting: "等待扫码确认...",
  wechat_qr_success: "微信登录成功",
  kind_label: "助手类型",
});

const els = {
  body: document.body,
  token: document.getElementById("ws-token"),
  connectBtn: document.getElementById("connect-btn"),
  disconnectBtn: document.getElementById("disconnect-btn"),
  sendTask: document.getElementById("send-task"),
  clearLog: document.getElementById("clear-log"),
  taskInput: document.getElementById("task-input"),
  log: document.getElementById("log-output"),
  socketState: document.getElementById("socket-state"),
  connectionBadge: document.getElementById("connection-badge"),
  connectionDetail: document.getElementById("connection-detail"),
  visionToggle: document.getElementById("vision-toggle"),
  settingsVisionToggle: document.getElementById("settings-vision-toggle"),
  webuiToggle: document.getElementById("webui-toggle"),
  skillToggle: document.getElementById("settings-skill-toggle"),
  visionBadge: document.getElementById("vision-badge"),
  themeToggle: document.getElementById("theme-toggle"),
  languageToggle: document.getElementById("language-toggle"),
  resultCard: document.getElementById("result-card"),
  screenshotCard: document.getElementById("screenshot-card"),
  screenshotPreview: document.getElementById("screenshot-preview"),
  channelGrid: document.getElementById("channel-grid"),
  channelsPanel: document.getElementById("channels"),
  refreshChannels: document.getElementById("refresh-channels"),
  refreshDevices: document.getElementById("refresh-devices"),
  deviceList: document.getElementById("device-list"),
  voiceSeconds: document.getElementById("voice-seconds"),
  voiceRecord: document.getElementById("voice-record"),
  voiceResult: document.getElementById("voice-result"),
  aiGroups: document.getElementById("ai-groups"),
  navItems: [...document.querySelectorAll(".nav-item")],
};

function t(key) {
  return dict[state.language][key] ?? dict.en[key] ?? key;
}

function loadGroups() {
  try {
    const parsed = JSON.parse(localStorage.getItem(storageKeys.groups) || "[]");
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function defaultGroupHapiEndpoint() {
  return "http://127.0.0.1:8766";
}

function saveGroups() {
  for (const group of state.groups) {
    if (Array.isArray(group.messages) && group.messages.length > MAX_GROUP_MESSAGES) {
      group.messages = group.messages.slice(-MAX_GROUP_MESSAGES);
    }
  }
  localStorage.setItem(storageKeys.groups, JSON.stringify(state.groups));
  localStorage.setItem(storageKeys.activeGroup, state.activeGroupId || "");
}

async function loadAiGroupFromBackend() {
  try {
    const response = await fetch("/api/ai-group", { cache: "no-store" });
    if (!response.ok) {
      throw new Error(`${response.status} ${response.statusText}`);
    }
    const payload = await response.json();
    const group = groupFromPayload(payload);
    state.groups = group ? [group] : [];
    state.activeGroupId = group?.id || "";
    saveGroups();
  } catch (error) {
    logLine(t("settings_title"), `${t("config_load_failed")}: ${error}`, "error");
  }
}

async function saveActiveGroupToBackend() {
  const group = activeGroup();
  if (!group) return;

  const response = await fetch("/api/ai-group", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payloadFromGroup(group)),
  });

  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText}`);
  }

  const payload = await response.json();
  const synced = groupFromPayload(payload);
  state.groups = synced ? [synced] : [];
  state.activeGroupId = synced?.id || "";
  saveGroups();
}

function groupFromPayload(payload) {
  if (!payload?.name?.trim()) return null;
  const existing = activeGroup();
  return {
    id: "desktop-config",
    name: payload.name.trim(),
    enabled: !!payload.enabled,
    hapiEndpoint: payload.hapi_endpoint || defaultGroupHapiEndpoint(),
    strategy: payload.strategy || "broadcast",
    people: (payload.people || []).map((member) => ({
      name: member.name || "Member",
      role: member.role || "Member",
      note: member.endpoint || t("local_member_note"),
    })),
    agents: (payload.agents || []).map((agent) => ({
      name: agent.name || "AI",
      role: agent.role || "AI",
      kind: agent.kind || "codex",
      endpoint: agent.endpoint || "",
    })),
    messages: existing?.messages || [],
  };
}

function payloadFromGroup(group) {
  return {
    enabled: group.enabled !== false,
    name: group.name,
    people: group.people.map((member) => ({
      name: member.name,
      role: member.role,
      endpoint: member.note || t("local_member_note"),
      kind: "person",
    })),
    agents: group.agents.map((agent) => ({
      name: agent.name,
      role: agent.role,
      endpoint: agent.endpoint || "",
      kind: agent.kind || "codex",
    })),
    hapi_endpoint: group.hapiEndpoint || defaultGroupHapiEndpoint(),
    strategy: group.strategy || "broadcast",
  };
}

function activeGroup() {
  return state.groups.find((group) => group.id === state.activeGroupId) || state.groups[0] || null;
}

function gatewayWsUrl() {
  const scheme = location.protocol === "https:" ? "wss:" : "ws:";
  return `${scheme}//${location.host}/ws`;
}

function setTheme(theme) {
  state.theme = theme;
  els.body.dataset.theme = theme;
  localStorage.setItem(storageKeys.theme, theme);
}

function applyTranslations() {
  for (const node of document.querySelectorAll("[data-i18n]")) {
    node.textContent = t(node.dataset.i18n);
  }
  for (const node of document.querySelectorAll("[data-i18n-placeholder]")) {
    node.placeholder = t(node.dataset.i18nPlaceholder);
  }
  els.languageToggle.textContent = state.language === "zh" ? "EN" : "中文";
  document.documentElement.lang = state.language === "zh" ? "zh-CN" : "en";
  updateConnectionUi();
  setVisionBadge();
  renderLogs();
}

function setLanguage(language) {
  state.language = language;
  localStorage.setItem(storageKeys.language, language);
  renderAiGroups();
  applyTranslations();
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function resolveLogValue(value) {
  if (value && typeof value === "object" && value.i18n) {
    return t(value.i18n);
  }
  return String(value ?? "");
}

function logLine(title, detail, tone = "info") {
  state.logs.unshift({
    time: new Date().toLocaleTimeString(),
    title,
    detail,
    tone,
  });
  if (state.logs.length > MAX_LOG_ENTRIES) {
    state.logs.length = MAX_LOG_ENTRIES;
  }
  renderLogs();
}

function renderLogs() {
  if (!els.log) return;

  if (!state.logs.length) {
    els.log.innerHTML = `
      <div class="empty-state log-empty">
        <strong>${escapeHtml(t("logs_empty_title"))}</strong>
        <p>${escapeHtml(t("logs_empty_body"))}</p>
      </div>
    `;
    return;
  }

  els.log.innerHTML = state.logs
    .map((entry) => `
      <div class="log-line ${entry.tone === "error" ? "is-error" : ""}">
        <small>${escapeHtml(entry.time)} | ${escapeHtml(resolveLogValue(entry.title))}</small>
        <code>${escapeHtml(resolveLogValue(entry.detail))}</code>
      </div>
    `)
    .join("");
}

function updateConnectionUi() {
  if (state.connected && state.authenticated) {
    els.socketState.textContent = t("online");
    els.socketState.className = "state-dot is-connected";
    els.connectionBadge.textContent = t("online");
    els.connectionDetail.textContent = t("connectedDetail");
    return;
  }
  if (state.connected) {
    els.socketState.textContent = t("open");
    els.socketState.className = "state-dot";
    els.connectionBadge.textContent = t("pending");
    els.connectionDetail.textContent = t("awaitingDetail");
    return;
  }
  els.socketState.textContent = t("idle");
  els.socketState.className = "state-dot is-idle";
  els.connectionBadge.textContent = t("offline");
  els.connectionDetail.textContent = t("waiting");
}

function setVisionBadge() {
  const enabled = els.visionToggle.checked;
  els.visionBadge.textContent = enabled ? t("vision") : t("command");
  if (els.settingsVisionToggle) {
    els.settingsVisionToggle.checked = enabled;
  }
}

function setResultCard(message, detail) {
  els.resultCard.innerHTML = `<strong>${escapeHtml(message)}</strong><p>${escapeHtml(detail)}</p>`;
}

function setVoiceResult(message, detail) {
  els.voiceResult.innerHTML = `<strong>${escapeHtml(message)}</strong><p>${escapeHtml(detail)}</p>`;
}

function setScreenshot(base64) {
  if (!base64) {
    els.screenshotCard.classList.add("is-empty");
    els.screenshotPreview.removeAttribute("src");
    return;
  }
  els.screenshotCard.classList.remove("is-empty");
  els.screenshotPreview.src = `data:image/png;base64,${base64}`;
}

function disconnectSocket(reason = t("manual")) {
  if (state.ws) {
    state.ws.close();
    state.ws = null;
  }
  state.connected = false;
  state.authenticated = false;
  updateConnectionUi();
  if (reason !== t("manual")) {
    logLine(t("socket"), reason, "error");
  }
}

function connectSocket() {
  if (state.ws && state.connected) {
    logLine(t("socket"), t("alreadyConnected"));
    return;
  }

  const url = gatewayWsUrl();
  const ws = new WebSocket(url);
  state.ws = ws;
  logLine(t("socket"), t("openingSocket")(url));

  ws.addEventListener("open", () => {
    state.connected = true;
    state.authenticated = false;
    updateConnectionUi();
    const authPayload = { type: "auth", token: els.token.value.trim() };
    ws.send(JSON.stringify(authPayload));
    logLine(t("auth"), JSON.stringify(authPayload, null, 2));
  });

  ws.addEventListener("message", (event) => {
    let parsed;
    try {
      parsed = JSON.parse(event.data);
    } catch {
      logLine("Message", event.data, "error");
      return;
    }

    if (parsed.type === "auth_result") {
      state.authenticated = !!parsed.success;
      updateConnectionUi();
    }

    if (parsed.type === "result") {
      const transcript = parsed.data?.transcript || [];
      setResultCard(parsed.message || t("noResult"), transcript.join("\n"));
      setScreenshot(parsed.data?.screenshot_base64 || "");
    }

    logLine(
      parsed.type || "message",
      JSON.stringify(parsed, null, 2),
      parsed.status === "error" ? "error" : "info",
    );
  });

  ws.addEventListener("close", () => disconnectSocket(t("socketClosed")));
  ws.addEventListener("error", () => {
    els.socketState.textContent = t("error");
    els.socketState.className = "state-dot is-error";
    logLine(t("socket"), t("socketError"), "error");
  });
}

function sendTask() {
  const task = els.taskInput.value.trim();
  if (!task) {
    logLine(t("task"), t("taskEmpty"), "error");
    return;
  }
  if (!state.ws || !state.connected) {
    logLine(t("task"), t("connectError"), "error");
    return;
  }
  const payload = { type: "task", task };
  state.ws.send(JSON.stringify(payload));
  logLine(t("task"), JSON.stringify(payload, null, 2));
}

async function loadChannels() {
  state.channelsLoaded = true;
  state.channelMode = "list";
  els.channelGrid.innerHTML = `<div class="empty-state">${t("channelsLoading")}</div>`;
  try {
    const configResponse = await fetch("/api/channel-config", { cache: "no-store" });
    state.channelConfig = configResponse.ok ? await configResponse.json() : {};
    const response = await fetch("/api/channels");
    const payload = await response.json();
    const cards = (payload.channels || []).map(
      (channel) => `
        <article class="channel-card">
          <div class="channel-top">
            <strong>${escapeHtml(channel.name || "Channel")}</strong>
            <span class="badge badge-${escapeHtml(channel.status || "disabled")}">${escapeHtml(channel.status || "")}</span>
          </div>
          <p>${escapeHtml(channel.detail || "")}</p>
        </article>
      `,
    );
    els.channelGrid.innerHTML = `
      <p class="section-hint">${escapeHtml(t("channels_subtitle"))}</p>
      <div class="channel-grid-inner">${cards.join("") || `<div class="empty-state">${t("noChannels")}</div>`}</div>
      <button id="channel-add" class="floating-add" type="button" title="${escapeHtml(t("add_channel"))}">+</button>
    `;
    document.getElementById("channel-add")?.addEventListener("click", showChannelCreate);
  } catch (error) {
    els.channelGrid.innerHTML = `<div class="empty-state">${escapeHtml(error)}</div>`;
  }
}

function showChannelCreate() {
  state.channelMode = "create";
  renderChannelCreate();
}

function renderChannelCreate() {
  const cfg = state.channelConfig || {};
  const kind = state.channelKind;
  els.channelGrid.innerHTML = `
    <div class="channel-create">
      <div class="channel-create-head">
        <button id="channel-back" class="ghost-button" type="button">${escapeHtml(t("back"))}</button>
        <div>
          <h4>${escapeHtml(t("create_channel"))}</h4>
          <p>${escapeHtml(t("channels_subtitle"))}</p>
        </div>
      </div>
      <label class="field-inline">
        <span>${escapeHtml(t("channel_type"))}</span>
        <select id="channel-kind">
          <option value="gateway">Web UI / WebSocket</option>
          <option value="wechat">WeChat iLink</option>
          <option value="wecom">WeCom</option>
          <option value="dingtalk">DingTalk</option>
          <option value="qq">QQ</option>
        </select>
      </label>
      <div id="channel-form" class="form-grid">${channelFormHtml(kind, cfg)}</div>
      <div class="inline-actions">
        <button id="channel-save" class="primary-button" type="button">${escapeHtml(t("save_channel"))}</button>
      </div>
    </div>
  `;
  document.getElementById("channel-kind").value = kind;
  const modeSelect = document.getElementById("channel-mode");
  if (modeSelect) modeSelect.value = (state.channelConfig || {}).qq_mode || "ws";
  document.getElementById("wechat-qr-login")?.addEventListener("click", startWechatQrLogin);
  document.getElementById("channel-kind").addEventListener("change", (event) => {
    state.channelKind = event.target.value;
    renderChannelCreate();
  });
  document.getElementById("channel-back")?.addEventListener("click", loadChannels);
  document.getElementById("channel-save")?.addEventListener("click", saveChannelConfig);
}

function channelFormHtml(kind, cfg) {
  const checked = (value) => (value ? "checked" : "");
  const value = (key, fallback = "") => escapeHtml(cfg[key] ?? fallback);
  const enabledKey = {
    gateway: "ws_enabled",
    wechat: "wc_enabled",
    wecom: "wcom_enabled",
    dingtalk: "dt_enabled",
    qq: "qq_enabled",
  }[kind];
  const common = `
    <label class="toggle-line settings-card">
      <input id="channel-enabled" type="checkbox" ${checked(cfg[enabledKey])} />
      <span>${escapeHtml(t("settings_local"))} / ${escapeHtml(t("online"))}</span>
    </label>
  `;
  if (kind === "gateway") {
    return `${common}${field("host", "Host", value("ws_host", "0.0.0.0"))}${field("port", "Port", value("ws_port", 9178), "number")}${field("token", "Token", value("ws_token"))}`;
  }
  if (kind === "wechat") {
    return `${common}
      <div class="qr-login-card">
        <button id="wechat-qr-login" class="ghost-button" type="button">${escapeHtml(t("wechat_qr_login"))}</button>
        <span>${escapeHtml(state.wechatQrStatus || t("wechat_qr_hint"))}</span>
        ${state.wechatQrImage ? `<img src="${escapeHtml(state.wechatQrImage)}" alt="WeChat QR" />` : ""}
      </div>
      ${field("token", "Bot Token", value("wc_token"))}`;
  }
  if (kind === "wecom") {
    return `${common}${field("corp_id", "Corp ID", value("wcom_corp_id"))}${field("agent_id", "Agent ID", value("wcom_agent_id"))}${field("secret", "Secret", value("wcom_secret"))}${field("token", "Token", value("wcom_token"))}${field("aes_key", "AES Key", value("wcom_aes_key"))}`;
  }
  if (kind === "dingtalk") {
    return `${common}${field("app_key", "App Key", value("dt_app_key"))}${field("app_secret", "App Secret", value("dt_app_secret"))}${field("webhook", "Webhook", value("dt_webhook"))}`;
  }
  return `${common}
    <label class="field-inline"><span>Mode</span><select id="channel-mode"><option value="ws">go-cqhttp WebSocket</option><option value="official">QQ Official Bot</option></select></label>
    ${field("ws_host", "WebSocket Host", value("qq_ws_host", "127.0.0.1"))}
    ${field("ws_port", "WebSocket Port", value("qq_ws_port", 6700), "number")}
    ${field("bot_appid", "Bot AppID", value("qq_bot_appid"))}
    ${field("bot_token", "Bot Token", value("qq_bot_token"))}`;
}

function field(id, label, value, type = "text") {
  return `<label class="field-inline"><span>${escapeHtml(label)}</span><input id="channel-${id}" type="${type}" value="${value}" /></label>`;
}

async function saveChannelConfig() {
  const kind = state.channelKind;
  const payload = { kind, enabled: document.getElementById("channel-enabled")?.checked || false };
  for (const input of document.querySelectorAll("#channel-form input")) {
    if (input.id === "channel-enabled") continue;
    const key = input.id.replace("channel-", "");
    payload[key] = input.type === "number" ? Number(input.value || 0) : input.value;
  }
  const qqMode = document.getElementById("channel-mode");
  if (qqMode) payload.mode = qqMode.value;

  try {
    const response = await fetch("/api/channel-config", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
    logLine(t("channels_title"), t("channel_saved"));
    await loadChannels();
  } catch (error) {
    logLine(t("channels_title"), String(error), "error");
  }
}

async function startWechatQrLogin() {
  if (state.wechatQrTimer) {
    clearInterval(state.wechatQrTimer);
    state.wechatQrTimer = null;
  }
  state.wechatQrPolling = false;
  state.wechatQrStatus = t("wechat_qr_loading");
  renderChannelCreate();

  try {
    const response = await fetch("/api/wechat/qr-login", { method: "POST" });
    const payload = await response.json();
    if (!response.ok) throw new Error(payload.error || `${response.status} ${response.statusText}`);

    state.wechatQrKey = payload.key || "";
    state.wechatQrImage = payload.image_data_url || "";
    state.wechatQrStatus = t("wechat_qr_waiting");
    renderChannelCreate();
    pollWechatQrStatus();
    state.wechatQrTimer = setInterval(pollWechatQrStatus, 5000);
  } catch (error) {
    state.wechatQrStatus = String(error);
    logLine("WeChat iLink", String(error), "error");
    renderChannelCreate();
  }
}

async function pollWechatQrStatus() {
  if (!state.wechatQrKey) return;
  if (state.wechatQrPolling) return;
  state.wechatQrPolling = true;

  try {
    const response = await fetch(`/api/wechat/qr-status?key=${encodeURIComponent(state.wechatQrKey)}`, {
      cache: "no-store",
    });
    const payload = await response.json();
    if (!response.ok) throw new Error(payload.error || `${response.status} ${response.statusText}`);

    if (payload.status === "confirmed") {
      if (state.wechatQrTimer) {
        clearInterval(state.wechatQrTimer);
        state.wechatQrTimer = null;
      }
      state.wechatQrStatus = t("wechat_qr_success");
      state.channelConfig.wc_enabled = true;
      state.channelConfig.wc_token = payload.token || state.channelConfig.wc_token || "";
      logLine("WeChat iLink", t("wechat_qr_success"));
      renderChannelCreate();
    } else if (payload.status === "expired") {
      if (state.wechatQrTimer) {
        clearInterval(state.wechatQrTimer);
        state.wechatQrTimer = null;
      }
      state.wechatQrStatus = "QR code expired";
      renderChannelCreate();
    }
  } catch (error) {
    if (state.wechatQrTimer) {
      clearInterval(state.wechatQrTimer);
      state.wechatQrTimer = null;
    }
    state.wechatQrStatus = String(error);
    logLine("WeChat iLink", String(error), "error");
    renderChannelCreate();
  } finally {
    state.wechatQrPolling = false;
  }
}

async function loadVoiceDevices() {
  state.voiceLoaded = true;
  els.deviceList.innerHTML = `<li>${t("devicesLoading")}</li>`;
  try {
    const response = await fetch("/api/voice/devices");
    const payload = await response.json();
    const devices = payload.devices || [];
    els.deviceList.innerHTML =
      devices
        .map((device) => `<li>${escapeHtml(device.name)}${device.default ? ` | ${t("defaultDevice")}` : ""}</li>`)
        .join("") || `<li>${t("noDevices")}</li>`;
  } catch (error) {
    els.deviceList.innerHTML = `<li>${escapeHtml(error)}</li>`;
  }
}

async function transcribeVoice() {
  const seconds = Number(els.voiceSeconds.value || 4);
  setVoiceResult(t("voiceRecordingTitle"), t("voiceRecordingBody")(seconds));
  try {
    const response = await fetch("/api/voice/transcribe", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ seconds }),
    });
    const payload = await response.json();
    if (payload.error) {
      throw new Error(payload.error);
    }
    setVoiceResult(payload.result.text || t("noVoice"), `sample rate ${payload.result.sample_rate}`);
    logLine(t("voice"), JSON.stringify(payload.result, null, 2));
  } catch (error) {
    setVoiceResult(t("voiceErrorTitle"), String(error));
    logLine(t("voice"), String(error), "error");
  }
}

function bindNavigation() {
  const views = {
    dashboard: ["dashboard", "metrics", "task-panel", "result-panel"],
    "ai-groups": ["ai-groups"],
    settings: ["settings"],
    channels: ["channels"],
    voice: ["voice"],
    logs: ["logs"],
  };
  const allViewKeys = [...new Set(Object.values(views).flat())];
  const resolveView = (key) => {
    if (key === "metrics") return document.querySelector(".metrics");
    if (key === "task-panel") return document.querySelector(".task-panel");
    if (key === "result-panel") return document.querySelector(".result-panel");
    return document.getElementById(key);
  };
  const showSection = (section) => {
    const active = new Set(views[section] || views.dashboard);
    for (const key of allViewKeys) {
      resolveView(key)?.classList.toggle("is-hidden", !active.has(key));
    }
    els.navItems.forEach((entry) => {
      entry.classList.toggle("is-active", entry.dataset.section === section);
    });
    if (section === "channels" && !state.channelsLoaded) {
      void loadChannels();
    }
    if (section === "voice" && !state.voiceLoaded) {
      void loadVoiceDevices();
    }
    window.scrollTo({ top: 0, behavior: "auto" });
  };

  for (const item of els.navItems) {
    item.addEventListener("click", () => showSection(item.dataset.section || "dashboard"));
  }

  showSection("dashboard");
}

function renderAiGroups() {
  const group = activeGroup();
  if (!group) {
    els.aiGroups.innerHTML = `
      <div class="group-empty">
        <div class="avatar-stack"><span>AI</span><span>+</span></div>
        <h3>${escapeHtml(t("ai_empty_title"))}</h3>
        <p>${escapeHtml(t("ai_empty_body"))}</p>
        <label class="field-inline">
          <span>${escapeHtml(t("ai_group_name_label"))}</span>
          <input id="new-group-name" type="text" placeholder="${escapeHtml(t("ai_group_name_placeholder"))}" />
        </label>
        <button id="create-group" class="primary-button" type="button">${escapeHtml(t("create_group"))}</button>
      </div>
    `;
    document.getElementById("create-group")?.addEventListener("click", createGroupFromInput);
    return;
  }

  state.activeGroupId = group.id;
  saveGroups();

  if (state.aiGroupMode === "person") {
    renderMemberForm(group);
    return;
  }
  if (state.aiGroupMode === "agent") {
    renderAgentForm(group);
    return;
  }

  const groupButtons = state.groups
    .map(
      (item) => `
        <button class="room-button ${item.id === group.id ? "is-active" : ""}" data-group-id="${escapeHtml(item.id)}">
          <strong>${escapeHtml(item.name)}</strong>
          <span>${item.people.length + item.agents.length} ${escapeHtml(t("people_title"))}</span>
        </button>
      `,
    )
    .join("");
  const people = group.people
    .map((member, index) => memberRow(member, "avatar-human", "person", index))
    .join("") || `<div class="empty-state">${escapeHtml(t("no_group_members"))}</div>`;
  const agents = group.agents
    .map((member, index) => memberRow(member, "avatar-code", "agent", index))
    .join("") || `<div class="empty-state">${escapeHtml(t("no_group_agents"))}</div>`;
  const messages = renderGroupMessages(group);

  els.aiGroups.innerHTML = `
    <div class="group-chat">
      <div class="group-chat-head">
        <div>
          <h3>${escapeHtml(group.name)}</h3>
          <p>${escapeHtml(t("group_desc"))}</p>
        </div>
        <div class="group-tools">
          <button class="icon-button" id="new-group" type="button">+</button>
          <button class="icon-button" id="edit-group" type="button">E</button>
        </div>
      </div>
      <div class="message-stream" id="group-message-stream">${messages}</div>
      <div class="group-composer">
        <button class="icon-button" id="composer-add-ai" type="button">+</button>
        <input id="group-task-input" placeholder="${escapeHtml(t("ai_placeholder"))}" ${state.aiGroupRunning ? "disabled" : ""} />
        <button class="primary-button" id="group-send" type="button" ${state.aiGroupRunning ? "disabled" : ""}>${escapeHtml(state.aiGroupRunning ? t("collaboration_running") : t("collaborate_button"))}</button>
      </div>
    </div>
    <aside class="group-settings">
      <div class="group-cover">
        <div class="avatar-stack"><span>${escapeHtml(group.name.slice(0, 1).toUpperCase())}</span><span>AI</span></div>
        <h3>${escapeHtml(group.name)}</h3>
        <p>${escapeHtml(t("group_desc"))}</p>
      </div>
      <div class="quick-actions">
        <button id="add-member" type="button"><strong>note</strong><span>${escapeHtml(t("add_member"))}</span></button>
        <button id="add-ai" type="button"><strong>AI</strong><span>${escapeHtml(t("add_ai"))}</span></button>
        <button id="edit-group-side" type="button"><strong>edit</strong><span>${escapeHtml(t("edit_group"))}</span></button>
      </div>
      <h4>${escapeHtml(t("group_list_title"))}</h4>
      <div class="room-list">${groupButtons}</div>
      <h4>${escapeHtml(t("people_title"))}</h4>
      <div class="member-list">${people}</div>
      <h4>${escapeHtml(t("ai_title"))}</h4>
      <div class="member-list">${agents}</div>
    </aside>
  `;

  for (const button of els.aiGroups.querySelectorAll(".room-button")) {
    button.addEventListener("click", () => {
      state.activeGroupId = button.dataset.groupId;
      renderAiGroups();
    });
  }
  document.getElementById("new-group")?.addEventListener("click", createGroupPrompt);
  document.getElementById("add-member")?.addEventListener("click", () => openMemberForm());
  document.getElementById("add-ai")?.addEventListener("click", () => openAgentForm());
  document.getElementById("composer-add-ai")?.addEventListener("click", () => openAgentForm());
  document.getElementById("group-send")?.addEventListener("click", sendGroupCollaboration);
  document.getElementById("group-task-input")?.addEventListener("keydown", (event) => {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      sendGroupCollaboration();
    }
  });
  document.getElementById("edit-group")?.addEventListener("click", editGroupPrompt);
  document.getElementById("edit-group-side")?.addEventListener("click", editGroupPrompt);
  for (const row of els.aiGroups.querySelectorAll(".member-row[data-member-type='person']")) {
    row.addEventListener("click", () => openMemberForm(Number(row.dataset.memberIndex)));
  }
  for (const row of els.aiGroups.querySelectorAll(".member-row[data-member-type='agent']")) {
    row.addEventListener("click", () => openAgentForm(Number(row.dataset.memberIndex)));
  }
}

function renderGroupMessages(group) {
  const messages = group.messages?.length
    ? group.messages
    : [
        {
          speaker: t("system_member"),
          role: t("group_settings"),
          kind: "system",
          status: "success",
          content: t("system_message"),
        },
      ];

  return messages
    .map((message) => {
      const avatarClass =
        message.kind === "person" || message.kind === "user"
          ? "avatar-human"
          : message.kind === "system"
            ? "avatar-warm"
            : "avatar-code";
      const statusClass = message.status === "error" ? " is-error" : "";
      const initials = (message.speaker || "AI").slice(0, 2).toUpperCase();
      return `
        <article class="chat-message${statusClass}">
          <div class="avatar ${avatarClass}">${escapeHtml(initials)}</div>
          <div>
            <div class="message-meta">
              <strong>${escapeHtml(message.speaker || "AI")}</strong>
              <span>${escapeHtml(message.role || message.kind || "member")}</span>
            </div>
            <p>${escapeHtml(message.content || "")}</p>
          </div>
        </article>
      `;
    })
    .join("");
}

async function sendGroupCollaboration() {
  const group = activeGroup();
  if (!group || state.aiGroupRunning) return;

  const input = document.getElementById("group-task-input");
  const task = (input?.value || "").trim();
  if (!task) {
    logLine(t("ai_title"), t("collaboration_empty"), "error");
    return;
  }

  group.messages = group.messages || [];
  group.messages.push({
    speaker: t("user_member"),
    role: "request",
    kind: "person",
    status: "input",
    content: task,
  });
  input.value = "";
  state.aiGroupRunning = true;
  saveGroups();
  renderAiGroups();

  try {
    await saveActiveGroupToBackend();
    const response = await fetch("/api/ai-group/collaborate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ task }),
    });
    const payload = await response.json();
    if (!response.ok || payload.error) {
      throw new Error(payload.error || `${response.status} ${response.statusText}`);
    }

    const returned = Array.isArray(payload.messages) ? payload.messages : [];
    for (const message of returned) {
      if (message.kind === "person" && message.status === "input") continue;
      group.messages.push({
        speaker: message.speaker || "AI",
        role: message.role || message.kind || "member",
        kind: message.kind || "ai",
        status: message.status || "success",
        content: message.content || "",
      });
    }
    logLine(t("ai_title"), payload.summary || t("group_updated"));
  } catch (error) {
    group.messages.push({
      speaker: t("system_member"),
      role: "coordinator",
      kind: "system",
      status: "error",
      content: `${t("collaboration_error")}: ${error}`,
    });
    logLine(t("ai_title"), `${t("collaboration_error")}: ${error}`, "error");
  } finally {
    state.aiGroupRunning = false;
    saveGroups();
    renderAiGroups();
  }
}

function memberRow(member, avatarClass, type, index) {
  const detail =
    type === "agent"
      ? `${member.kind || "ai"} | ${member.endpoint || "no endpoint"}`
      : member.note || t("local_member_note");
  return `
    <div class="member-row" data-member-type="${escapeHtml(type)}" data-member-index="${index}">
      <span class="avatar small ${avatarClass}">${escapeHtml(member.name.slice(0, 1).toUpperCase())}</span>
      <strong>${escapeHtml(member.name)}</strong>
      <em title="${escapeHtml(detail)}">${escapeHtml(member.role)}</em>
    </div>
  `;
}

function openMemberForm(index = null) {
  state.aiGroupMode = "person";
  state.aiGroupEditIndex = Number.isInteger(index) ? index : null;
  renderAiGroups();
}

function openAgentForm(index = null) {
  state.aiGroupMode = "agent";
  state.aiGroupEditIndex = Number.isInteger(index) ? index : null;
  renderAiGroups();
}

function closeAiGroupForm() {
  state.aiGroupMode = "chat";
  state.aiGroupEditIndex = null;
  renderAiGroups();
}

function renderMemberForm(group) {
  const index = state.aiGroupEditIndex;
  const member = Number.isInteger(index) ? group.people[index] : null;
  els.aiGroups.innerHTML = `
    <div class="channel-create group-form">
      <div class="channel-create-head">
        <button class="ghost-button" id="group-form-back" type="button">${escapeHtml(t("back"))}</button>
        <div>
          <h4>${escapeHtml(t("member_form_title"))}</h4>
          <p>${escapeHtml(t("local_member_note"))}</p>
        </div>
      </div>
      <div class="form-grid group-form-card">
        <label>
          <span>${escapeHtml(t("member_name_prompt"))}</span>
          <input id="member-name" type="text" value="${escapeHtml(member?.name || "")}" />
        </label>
        <label>
          <span>${escapeHtml(t("member_role_prompt"))}</span>
          <input id="member-role" type="text" value="${escapeHtml(member?.role || "Member")}" />
        </label>
        <label>
          <span>${escapeHtml(t("note_label"))}</span>
          <input id="member-note" type="text" value="${escapeHtml(member?.note || t("local_member_note"))}" />
        </label>
        <div class="inline-actions">
          <button class="primary-button" id="save-member-form" type="button">${escapeHtml(t("save_member"))}</button>
          <button class="ghost-button" id="cancel-member-form" type="button">${escapeHtml(t("cancel"))}</button>
        </div>
      </div>
    </div>
  `;
  document.getElementById("group-form-back")?.addEventListener("click", closeAiGroupForm);
  document.getElementById("cancel-member-form")?.addEventListener("click", closeAiGroupForm);
  document.getElementById("save-member-form")?.addEventListener("click", saveMemberForm);
}

function renderAgentForm(group) {
  const index = state.aiGroupEditIndex;
  const agent = Number.isInteger(index) ? group.agents[index] : null;
  const kind = normalizeAgentKind(agent?.kind || "codex");
  const defaultEndpoint = defaultAgentEndpoint(kind);
  const endpointLabel = agentEndpointLabel(kind);
  const options = ["codex", "openclaw", "claude", "opencode", "astrbot", "api"]
    .map((value) => `<option value="${value}" ${value === kind ? "selected" : ""}>${value}</option>`)
    .join("");

  els.aiGroups.innerHTML = `
    <div class="channel-create group-form">
      <div class="channel-create-head">
        <button class="ghost-button" id="group-form-back" type="button">${escapeHtml(t("back"))}</button>
        <div>
          <h4>${escapeHtml(t("ai_form_title"))}</h4>
          <p>${escapeHtml(t("mixed_agent_hint"))}</p>
        </div>
      </div>
      <div class="form-grid group-form-card">
        <label>
          <span>${escapeHtml(t("ai_name_prompt"))}</span>
          <input id="agent-name" type="text" value="${escapeHtml(agent?.name || "Codex")}" />
        </label>
        <label>
          <span>${escapeHtml(t("ai_role_prompt"))}</span>
          <input id="agent-role" type="text" value="${escapeHtml(agent?.role || "Implementer")}" />
        </label>
        <label>
          <span>${escapeHtml(t("kind_label"))}</span>
          <select id="agent-kind">${options}</select>
        </label>
        <label>
          <span id="agent-endpoint-label">${escapeHtml(endpointLabel)}</span>
          <input id="agent-endpoint" type="text" value="${escapeHtml(agent?.endpoint || defaultEndpoint)}" />
        </label>
        <div class="inline-actions">
          <button class="primary-button" id="save-agent-form" type="button">${escapeHtml(t("save_ai"))}</button>
          <button class="ghost-button" id="cancel-agent-form" type="button">${escapeHtml(t("cancel"))}</button>
        </div>
      </div>
    </div>
  `;
  document.getElementById("group-form-back")?.addEventListener("click", closeAiGroupForm);
  document.getElementById("cancel-agent-form")?.addEventListener("click", closeAiGroupForm);
  document.getElementById("save-agent-form")?.addEventListener("click", saveAgentForm);
  document.getElementById("agent-kind")?.addEventListener("change", updateAgentEndpointHint);
}

async function saveMemberForm() {
  const group = activeGroup();
  if (!group) return;
  const name = document.getElementById("member-name")?.value.trim() || "";
  if (!name) return;
  const role = document.getElementById("member-role")?.value.trim() || "Member";
  const note = document.getElementById("member-note")?.value.trim() || t("local_member_note");
  const member = { name, role, note };
  if (Number.isInteger(state.aiGroupEditIndex) && group.people[state.aiGroupEditIndex]) {
    group.people[state.aiGroupEditIndex] = member;
  } else {
    group.people.push(member);
  }
  state.aiGroupMode = "chat";
  state.aiGroupEditIndex = null;
  await saveAndRender(t("member_added"), `${name} / ${role}`);
}

async function saveAgentForm() {
  const group = activeGroup();
  if (!group) return;
  const name = document.getElementById("agent-name")?.value.trim() || "";
  if (!name) return;
  const role = document.getElementById("agent-role")?.value.trim() || "AI";
  const kind = normalizeAgentKind(document.getElementById("agent-kind")?.value || "codex");
  const endpoint = document.getElementById("agent-endpoint")?.value.trim() || "";
  if (!endpoint) return;
  const agent = { name, role, kind, endpoint };
  if (Number.isInteger(state.aiGroupEditIndex) && group.agents[state.aiGroupEditIndex]) {
    group.agents[state.aiGroupEditIndex] = agent;
  } else {
    group.agents.push(agent);
  }
  state.aiGroupMode = "chat";
  state.aiGroupEditIndex = null;
  await saveAndRender(t("ai_added"), `${name} / ${role} / ${kind}`);
}

async function createGroupFromInput() {
  const input = document.getElementById("new-group-name");
  await createGroup((input?.value || "").trim());
}

async function createGroupPrompt() {
  await createGroup(window.prompt(t("group_name_prompt"), "") || "");
}

async function createGroup(name) {
  const trimmed = name.trim();
  if (!trimmed) return;
  const group = {
    id: "desktop-config",
    name: trimmed,
    enabled: true,
    hapiEndpoint: defaultGroupHapiEndpoint(),
    strategy: "broadcast",
    people: [],
    agents: [],
    messages: [],
  };
  state.groups = [group];
  state.activeGroupId = group.id;
  saveGroups();
  await saveAndRender(t("group_created"), trimmed);
}

async function editGroupPrompt() {
  const group = activeGroup();
  if (!group) return;
  const name = (window.prompt(t("group_name_prompt"), group.name) || "").trim();
  if (!name) return;
  group.name = name;
  group.hapiEndpoint = (window.prompt("HAPI endpoint", group.hapiEndpoint || defaultGroupHapiEndpoint()) || group.hapiEndpoint || "").trim();
  group.strategy = (window.prompt("strategy", group.strategy || "broadcast") || group.strategy || "broadcast").trim();
  await saveAndRender(t("group_updated"), name);
}

async function addMemberPrompt() {
  const group = activeGroup();
  if (!group) return;
  const name = (window.prompt(t("member_name_prompt"), "") || "").trim();
  if (!name) return;
  const role = (window.prompt(t("member_role_prompt"), "Member") || "Member").trim();
  const note = (window.prompt(t("local_member_note"), t("local_member_note")) || t("local_member_note")).trim();
  group.people.push({ name, role, note });
  await saveAndRender(t("member_added"), `${name} | ${role}`);
}

async function addAiPrompt() {
  const group = activeGroup();
  if (!group) return;
  const name = (window.prompt(t("ai_name_prompt"), "Codex") || "").trim();
  if (!name) return;
  const role = (window.prompt(t("ai_role_prompt"), "Implementer") || "AI").trim();
  const kind = normalizeAgentKind(window.prompt(t("ai_kind_prompt"), "codex") || "codex");
  const endpoint = (window.prompt(t("ai_endpoint_prompt"), defaultAgentEndpoint(kind)) || "").trim();
  group.agents.push({ name, role, kind, endpoint });
  await saveAndRender(t("ai_added"), `${name} | ${role} | ${kind} | ${endpoint}`);
}

async function editMemberPrompt(index) {
  const group = activeGroup();
  const member = group?.people[index];
  if (!group || !member) return;
  const name = (window.prompt(t("member_name_prompt"), member.name) || "").trim();
  if (!name) return;
  member.name = name;
  member.role = (window.prompt(t("member_role_prompt"), member.role) || member.role).trim();
  member.note = (window.prompt(t("local_member_note"), member.note || t("local_member_note")) || member.note || "").trim();
  await saveAndRender(t("member_added"), `${member.name} | ${member.role}`);
}

async function editAiPrompt(index) {
  const group = activeGroup();
  const agent = group?.agents[index];
  if (!group || !agent) return;
  const name = (window.prompt(t("ai_name_prompt"), agent.name) || "").trim();
  if (!name) return;
  agent.name = name;
  agent.role = (window.prompt(t("ai_role_prompt"), agent.role) || agent.role).trim();
  agent.kind = normalizeAgentKind(window.prompt(t("ai_kind_prompt"), agent.kind || "codex") || agent.kind || "codex");
  agent.endpoint = (window.prompt(t("ai_endpoint_prompt"), agent.endpoint || defaultAgentEndpoint(agent.kind)) || agent.endpoint || "").trim();
  await saveAndRender(t("ai_added"), `${agent.name} | ${agent.role} | ${agent.kind} | ${agent.endpoint}`);
}

function normalizeAgentKind(value) {
  const normalized = String(value).trim().toLowerCase();
  if (normalized.includes("api") || normalized.includes("openai")) return "api";
  if (normalized.includes("claw")) return "openclaw";
  if (normalized.includes("claude")) return "claude";
  if (normalized.includes("astr")) return "astrbot";
  if (normalized.includes("open")) return "opencode";
  return normalized || "codex";
}

function agentUsesWs(kind) {
  return kind === "openclaw" || kind === "astrbot";
}

function defaultAgentEndpoint(kind) {
  switch (normalizeAgentKind(kind)) {
    case "openclaw":
      return "ws://127.0.0.1:3000";
    case "astrbot":
      return "ws://127.0.0.1:6185";
    case "opencode":
      return "builtin://opencode";
    case "claude":
      return "http://127.0.0.1:9103";
    case "api":
      return "openai:https://api.example.com/v1/chat/completions;model=gpt-4o-mini;key=sk-...";
    case "codex":
    default:
      return "builtin://codex";
  }
}

function isDefaultAgentEndpoint(value) {
  const trimmed = String(value || "").trim();
  return ["openclaw", "astrbot", "opencode", "claude", "api", "codex"].some(
    (kind) => trimmed === defaultAgentEndpoint(kind),
  );
}

function agentEndpointLabel(kind) {
  if (agentUsesWs(kind)) return t("ws_endpoint_label");
  if (normalizeAgentKind(kind) === "api") return t("api_endpoint_label") || "API Endpoint";
  return t("hapi_endpoint_label");
}

function updateAgentEndpointHint() {
  const kind = normalizeAgentKind(document.getElementById("agent-kind")?.value || "codex");
  const endpoint = document.getElementById("agent-endpoint");
  const label = document.getElementById("agent-endpoint-label");
  if (label) {
    label.textContent = agentEndpointLabel(kind);
  }
  if (
    endpoint &&
    (!endpoint.value.trim() ||
      isDefaultAgentEndpoint(endpoint.value) ||
      /^(wss?:\/\/127\.0\.0\.1|https?:\/\/127\.0\.0\.1)/.test(endpoint.value.trim()))
  ) {
    endpoint.value = defaultAgentEndpoint(kind);
  }
}

async function saveAndRender(title, detail) {
  saveGroups();
  try {
    await saveActiveGroupToBackend();
    logLine(title, `${detail}\n${t("config_saved")}`);
  } catch (error) {
    logLine(t("settings_title"), `${t("config_save_failed")}: ${error}`, "error");
  }
  renderAiGroups();
}

els.connectBtn.addEventListener("click", connectSocket);
els.disconnectBtn.addEventListener("click", () => disconnectSocket());
els.sendTask.addEventListener("click", sendTask);
els.clearLog.addEventListener("click", () => {
  state.logs = [];
  renderLogs();
});
els.visionToggle.addEventListener("change", setVisionBadge);
els.settingsVisionToggle?.addEventListener("change", () => {
  els.visionToggle.checked = els.settingsVisionToggle.checked;
  setVisionBadge();
});
els.webuiToggle?.addEventListener("change", () => {
  logLine(t("settings_title"), `${t("settings_webui_toggle")}: ${els.webuiToggle.checked ? t("online") : t("offline")}`);
});
els.skillToggle?.addEventListener("change", () => {
  logLine(t("settings_title"), `${t("settings_skill_toggle")}: ${els.skillToggle.checked ? t("online") : t("offline")}`);
});
els.themeToggle.addEventListener("click", () => setTheme(state.theme === "dark" ? "light" : "dark"));
els.languageToggle.addEventListener("click", () => setLanguage(state.language === "zh" ? "en" : "zh"));
els.refreshChannels.addEventListener("click", loadChannels);
els.refreshDevices.addEventListener("click", loadVoiceDevices);
els.voiceRecord.addEventListener("click", transcribeVoice);

bindNavigation();
setTheme(state.theme);
setScreenshot("");
setResultCard(t("result_empty_title"), t("result_empty_body"));
setVoiceResult(t("voice_empty_title"), t("voice_empty_body"));
logLine({ i18n: "bootTitle" }, { i18n: "boot" });
loadAiGroupFromBackend().finally(() => {
  renderAiGroups();
  setLanguage(state.language);
});
