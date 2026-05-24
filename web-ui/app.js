const storageKeys = {
  theme: "eyeforge-web-theme",
  language: "eyeforge-web-language",
  groups: "eyeforge-ai-groups",
  activeGroup: "eyeforge-active-ai-group",
};

const state = {
  ws: null,
  connected: false,
  authenticated: false,
  theme: localStorage.getItem(storageKeys.theme) || "dark",
  language: localStorage.getItem(storageKeys.language) || "zh",
  groups: loadGroups(),
  activeGroupId: localStorage.getItem(storageKeys.activeGroup) || "",
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
    refresh_button: "Refresh",
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
    add_member: "Add Member",
    add_ai: "Add AI",
    edit_group: "Edit Group",
    people_title: "People",
    ai_title: "AI",
    member_name_prompt: "Member name",
    member_role_prompt: "Member role",
    ai_name_prompt: "AI name",
    ai_role_prompt: "AI role",
    group_name_prompt: "Group name",
    group_created: "Group created",
    member_added: "Member added",
    ai_added: "AI added",
    group_updated: "Group updated",
    ai_placeholder: "Mention a member or type a goal",
    system_member: "Group Assistant",
    system_message: "The group is ready. Add members or AI agents, then route tasks by role here.",
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
    add_member: "添加成员",
    add_ai: "添加 AI",
    edit_group: "编辑群信息",
    people_title: "成员",
    ai_title: "AI",
    member_name_prompt: "成员名称",
    member_role_prompt: "成员角色",
    ai_name_prompt: "AI 名称",
    ai_role_prompt: "AI 角色",
    group_name_prompt: "群聊名称",
    group_created: "群聊已创建",
    member_added: "成员已添加",
    ai_added: "AI 已添加",
    group_updated: "群聊已更新",
    ai_placeholder: "@成员 或输入任务目标",
    system_member: "群组助手",
    system_message: "群聊已创建。添加成员或 AI 后，可以在这里按角色分配任务。",
  },
};

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

function saveGroups() {
  localStorage.setItem(storageKeys.groups, JSON.stringify(state.groups));
  localStorage.setItem(storageKeys.activeGroup, state.activeGroupId || "");
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

function logLine(title, detail, tone = "info") {
  const line = document.createElement("div");
  line.className = "log-line";
  line.innerHTML = `
    <small>${new Date().toLocaleTimeString()} | ${escapeHtml(title)}</small>
    <code>${escapeHtml(detail)}</code>
  `;
  if (tone === "error") {
    line.style.borderColor = "rgba(255, 123, 123, 0.45)";
  }
  els.log.prepend(line);
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
  els.channelGrid.innerHTML = `<div class="empty-state">${t("channelsLoading")}</div>`;
  try {
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
    els.channelGrid.innerHTML = cards.join("") || `<div class="empty-state">${t("noChannels")}</div>`;
  } catch (error) {
    els.channelGrid.innerHTML = `<div class="empty-state">${escapeHtml(error)}</div>`;
  }
}

async function loadVoiceDevices() {
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
    window.scrollTo({ top: 0, behavior: "smooth" });
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
    .map((member) => memberRow(member, "avatar-human"))
    .join("") || `<div class="empty-state">${escapeHtml(t("no_group_members"))}</div>`;
  const agents = group.agents
    .map((member) => memberRow(member, "avatar-code"))
    .join("") || `<div class="empty-state">${escapeHtml(t("no_group_agents"))}</div>`;

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
      <div class="message-stream">
        <article class="chat-message">
          <div class="avatar">AI</div>
          <div>
            <div class="message-meta"><strong>${escapeHtml(t("system_member"))}</strong><span>${escapeHtml(t("group_settings"))}</span></div>
            <p>${escapeHtml(t("system_message"))}</p>
          </div>
        </article>
      </div>
      <div class="group-composer">
        <button class="icon-button" id="composer-add-ai" type="button">+</button>
        <input placeholder="${escapeHtml(t("ai_placeholder"))}" />
      </div>
    </div>
    <aside class="group-settings">
      <div class="group-cover">
        <div class="avatar-stack"><span>${escapeHtml(group.name.slice(0, 1).toUpperCase())}</span><span>AI</span></div>
        <h3>${escapeHtml(group.name)}</h3>
        <p>${escapeHtml(t("group_desc"))}</p>
      </div>
      <div class="quick-actions">
        <button id="add-member" type="button"><strong>people</strong><span>${escapeHtml(t("add_member"))}</span></button>
        <button id="add-ai" type="button"><strong>+</strong><span>${escapeHtml(t("add_ai"))}</span></button>
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
  document.getElementById("add-member")?.addEventListener("click", addMemberPrompt);
  document.getElementById("add-ai")?.addEventListener("click", addAiPrompt);
  document.getElementById("composer-add-ai")?.addEventListener("click", addAiPrompt);
  document.getElementById("edit-group")?.addEventListener("click", editGroupPrompt);
  document.getElementById("edit-group-side")?.addEventListener("click", editGroupPrompt);
}

function memberRow(member, avatarClass) {
  return `
    <div class="member-row">
      <span class="avatar small ${avatarClass}">${escapeHtml(member.name.slice(0, 1).toUpperCase())}</span>
      <strong>${escapeHtml(member.name)}</strong>
      <em>${escapeHtml(member.role)}</em>
    </div>
  `;
}

function createGroupFromInput() {
  const input = document.getElementById("new-group-name");
  createGroup((input?.value || "").trim());
}

function createGroupPrompt() {
  createGroup(window.prompt(t("group_name_prompt"), "") || "");
}

function createGroup(name) {
  const trimmed = name.trim();
  if (!trimmed) return;
  const group = { id: crypto.randomUUID(), name: trimmed, people: [], agents: [] };
  state.groups.push(group);
  state.activeGroupId = group.id;
  saveGroups();
  renderAiGroups();
  logLine(t("group_created"), trimmed);
}

function editGroupPrompt() {
  const group = activeGroup();
  if (!group) return;
  const name = (window.prompt(t("group_name_prompt"), group.name) || "").trim();
  if (!name) return;
  group.name = name;
  saveGroups();
  renderAiGroups();
  logLine(t("group_updated"), name);
}

function addMemberPrompt() {
  const group = activeGroup();
  if (!group) return;
  const name = (window.prompt(t("member_name_prompt"), "") || "").trim();
  if (!name) return;
  const role = (window.prompt(t("member_role_prompt"), "Member") || "Member").trim();
  group.people.push({ name, role });
  saveGroups();
  renderAiGroups();
  logLine(t("member_added"), `${name} | ${role}`);
}

function addAiPrompt() {
  const group = activeGroup();
  if (!group) return;
  const name = (window.prompt(t("ai_name_prompt"), "Codex") || "").trim();
  if (!name) return;
  const role = (window.prompt(t("ai_role_prompt"), "Implementer") || "AI").trim();
  group.agents.push({ name, role });
  saveGroups();
  renderAiGroups();
  logLine(t("ai_added"), `${name} | ${role}`);
}

els.connectBtn.addEventListener("click", connectSocket);
els.disconnectBtn.addEventListener("click", () => disconnectSocket());
els.sendTask.addEventListener("click", sendTask);
els.clearLog.addEventListener("click", () => (els.log.innerHTML = ""));
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
renderAiGroups();
setLanguage(state.language);
setScreenshot("");
setResultCard(t("result_empty_title"), t("result_empty_body"));
setVoiceResult(t("voice_empty_title"), t("voice_empty_body"));
loadChannels();
loadVoiceDevices();
logLine(t("bootTitle"), t("boot"));
