const state = {
  ws: null,
  connected: false,
  authenticated: false,
  theme: localStorage.getItem("eyeforge-web-theme") || "dark",
  language: localStorage.getItem("eyeforge-web-language") || "en",
};

const dict = {
  en: {
    sidebar_subtitle: "Rust Gateway Console",
    sidebar_gateway: "Gateway :9178",
    nav_dashboard: "Dashboard",
    nav_channels: "Channels",
    nav_voice: "Voice",
    nav_logs: "Logs",
    hero_eyebrow: "Rust Native Gateway",
    hero_title: "Browser console for the EyeForge gateway",
    hero_body:
      "One port, one backend. The Rust gateway serves this UI on http://127.0.0.1:9178/, exposes WebSocket execution at /ws, and publishes channel and voice APIs too.",
    theme_button: "Theme",
    metric_gateway_label: "Gateway",
    metric_socket_label: "WebSocket",
    metric_socket_detail: "Task execution and structured actions share the same runtime.",
    metric_mode_label: "Mode",
    metric_mode_detail: "Visual hints only affect the UI. Real execution follows backend actions.",
    task_title: "Task Composer",
    task_vision_hint: "Vision hint",
    task_placeholder:
      "Examples:\nNatural language: Open Notepad and type a short note\nStructured JSON: {\"actions\":[{\"type\":\"wait\",\"seconds\":1},{\"type\":\"complete\",\"result\":\"ok\"}]}",
    connect_button: "Connect",
    disconnect_button: "Disconnect",
    run_button: "Run Task",
    token_label: "Token",
    token_placeholder: "Optional token",
    result_title: "Latest Result",
    result_empty_title: "No result yet",
    result_empty_body: "Run a task from the Rust gateway or use voice transcription below.",
    screenshot_empty: "Screenshot output will appear here when a task returns image data.",
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
    resultSeed: "Run a gateway task to populate this card.",
    connectError: "WebSocket is not connected",
    taskEmpty: "Task is empty",
    socketError: "WebSocket error",
    alreadyConnected: "Already connected",
    openingSocket: (url) => `Opening ${url}`,
  },
  zh: {
    sidebar_subtitle: "Rust 网关控制台",
    sidebar_gateway: "网关 :9178",
    nav_dashboard: "控制台",
    nav_channels: "通道",
    nav_voice: "语音",
    nav_logs: "日志",
    hero_eyebrow: "Rust 原生网关",
    hero_title: "浏览器里的 EyeForge 控制台",
    hero_body:
      "一个端口，一套后端。Rust 网关在 http://127.0.0.1:9178/ 托管这个页面，在 /ws 提供任务执行，并同时暴露通道和语音 API。",
    theme_button: "主题",
    metric_gateway_label: "网关",
    metric_socket_label: "WebSocket",
    metric_socket_detail: "任务执行和结构化动作共用同一条 Rust 运行链。",
    metric_mode_label: "模式",
    metric_mode_detail: "视觉提示只影响界面，真实执行以后端动作链为准。",
    task_title: "任务输入",
    task_vision_hint: "视觉提示",
    task_placeholder:
      "示例：\n自然语言：打开记事本并输入一段短句\n结构化 JSON：{\"actions\":[{\"type\":\"wait\",\"seconds\":1},{\"type\":\"complete\",\"result\":\"ok\"}]}",
    connect_button: "连接",
    disconnect_button: "断开",
    run_button: "执行任务",
    token_label: "令牌",
    token_placeholder: "可选令牌",
    result_title: "最新结果",
    result_empty_title: "还没有结果",
    result_empty_body: "运行一个 Rust 网关任务，或者在下方使用语音转写。",
    screenshot_empty: "当任务返回图像数据时，截图预览会显示在这里。",
    channels_title: "通道矩阵",
    refresh_button: "刷新",
    voice_title: "语音控制台",
    devices_button: "设备",
    voice_devices_title: "输入设备",
    voice_transcribe_title: "原生语音转写",
    voice_seconds_label: "录音秒数",
    voice_record_button: "录音并转写",
    voice_empty_title: "还没有转写结果",
    voice_empty_body: "Rust 后端会从默认麦克风录音，并在配置了支持的提供商后执行转写。",
    logs_title: "事件日志",
    clear_button: "清空",
    online: "已连接",
    pending: "待认证",
    offline: "未连接",
    waiting: "等待连接 Rust 网关",
    connectedDetail: "WebSocket 与认证均已通过",
    awaitingDetail: "连接已建立，正在等待认证结果",
    vision: "视觉模式",
    command: "命令模式",
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
    resultSeed: "运行一个网关任务后，这里会显示最新结果。",
    connectError: "WebSocket 尚未连接",
    taskEmpty: "任务内容不能为空",
    socketError: "WebSocket 连接错误",
    alreadyConnected: "已经连接",
    openingSocket: (url) => `正在连接 ${url}`,
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
  navItems: [...document.querySelectorAll(".nav-item")],
  i18nNodes: [...document.querySelectorAll("[data-i18n]")],
  i18nPlaceholders: [...document.querySelectorAll("[data-i18n-placeholder]")],
};

function t(key) {
  return dict[state.language][key];
}

function gatewayHttpUrl(path = "") {
  return `${location.origin}${path}`;
}

function gatewayWsUrl() {
  const scheme = location.protocol === "https:" ? "wss:" : "ws:";
  return `${scheme}//${location.host}/ws`;
}

function setTheme(theme) {
  state.theme = theme;
  els.body.dataset.theme = theme;
  localStorage.setItem("eyeforge-web-theme", theme);
}

function applyTranslations() {
  for (const node of els.i18nNodes) {
    node.textContent = t(node.dataset.i18n);
  }
  for (const node of els.i18nPlaceholders) {
    node.placeholder = t(node.dataset.i18nPlaceholder);
  }
  els.languageToggle.textContent = state.language === "en" ? "中文" : "EN";
  updateConnectionUi();
  setVisionBadge();
}

function setLanguage(language) {
  state.language = language;
  localStorage.setItem("eyeforge-web-language", language);
  applyTranslations();
}

function logLine(title, detail, tone = "info") {
  const line = document.createElement("div");
  line.className = "log-line";
  line.innerHTML = `
    <small>${new Date().toLocaleTimeString()} · ${title}</small>
    <code>${escapeHtml(detail)}</code>
  `;
  if (tone === "error") {
    line.style.borderColor = "rgba(255, 123, 123, 0.28)";
  }
  els.log.prepend(line);
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function updateConnectionUi() {
  if (state.connected && state.authenticated) {
    els.socketState.textContent = "Connected";
    els.socketState.className = "state-dot is-connected";
    els.connectionBadge.textContent = t("online");
    els.connectionDetail.textContent = t("connectedDetail");
    return;
  }
  if (state.connected) {
    els.socketState.textContent = "Open";
    els.socketState.className = "state-dot";
    els.connectionBadge.textContent = t("pending");
    els.connectionDetail.textContent = t("awaitingDetail");
    return;
  }
  els.socketState.textContent = "Idle";
  els.socketState.className = "state-dot is-idle";
  els.connectionBadge.textContent = t("offline");
  els.connectionDetail.textContent = t("waiting");
}

function setVisionBadge() {
  els.visionBadge.textContent = els.visionToggle.checked ? t("vision") : t("command");
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

function disconnectSocket(reason = "manual") {
  if (state.ws) {
    state.ws.close();
    state.ws = null;
  }
  state.connected = false;
  state.authenticated = false;
  updateConnectionUi();
  if (reason !== "manual") {
    logLine("Socket", reason, "error");
  }
}

function connectSocket() {
  if (state.ws && state.connected) {
    logLine("Socket", t("alreadyConnected"));
    return;
  }

  const url = gatewayWsUrl();
  const ws = new WebSocket(url);
  state.ws = ws;
  logLine("Socket", t("openingSocket")(url));

  ws.addEventListener("open", () => {
    state.connected = true;
    state.authenticated = false;
    updateConnectionUi();
    const authPayload = { type: "auth", token: els.token.value.trim() };
    ws.send(JSON.stringify(authPayload));
    logLine("Auth", JSON.stringify(authPayload, null, 2));
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

  ws.addEventListener("close", () => disconnectSocket("socket closed"));
  ws.addEventListener("error", () => {
    els.socketState.textContent = "Error";
    els.socketState.className = "state-dot is-error";
    logLine("Socket", t("socketError"), "error");
  });
}

function sendTask() {
  const task = els.taskInput.value.trim();
  if (!task) {
    logLine("Task", t("taskEmpty"), "error");
    return;
  }
  if (!state.ws || !state.connected) {
    logLine("Task", t("connectError"), "error");
    return;
  }
  const payload = { type: "task", task };
  state.ws.send(JSON.stringify(payload));
  logLine("Task", JSON.stringify(payload, null, 2));
}

async function loadChannels() {
  els.channelGrid.innerHTML = `<div class="empty-state">${t("channelsLoading")}</div>`;
  try {
    const response = await fetch(gatewayHttpUrl("/api/channels"));
    const payload = await response.json();
    const cards = (payload.channels || []).map(
      (channel) => `
        <article class="channel-card">
          <div class="channel-top">
            <strong>${escapeHtml(channel.label)}</strong>
            <span class="badge badge-${escapeHtml(channel.status)}">${escapeHtml(channel.status)}</span>
          </div>
          <p>${escapeHtml(channel.detail)}</p>
        </article>
      `,
    );
    els.channelGrid.innerHTML = cards.join("") || `<div class="empty-state">${t("noChannels")}</div>`;
  } catch (error) {
    els.channelGrid.innerHTML = `<div class="empty-state">${escapeHtml(String(error))}</div>`;
  }
}

async function loadVoiceDevices() {
  els.deviceList.innerHTML = `<li>${t("devicesLoading")}</li>`;
  try {
    const response = await fetch(gatewayHttpUrl("/api/voice/devices"));
    const payload = await response.json();
    const devices = payload.devices || [];
    els.deviceList.innerHTML = devices
      .map(
        (device) =>
          `<li>${escapeHtml(device.name)}${device.is_default ? ` <span>(${t("defaultDevice")})</span>` : ""}</li>`,
      )
      .join("");
    if (!devices.length) {
      els.deviceList.innerHTML = `<li>${t("noDevices")}</li>`;
    }
  } catch (error) {
    els.deviceList.innerHTML = `<li>${escapeHtml(String(error))}</li>`;
  }
}

async function transcribeVoice() {
  const seconds = Math.max(1, Math.min(15, Number(els.voiceSeconds.value || 4)));
  setVoiceResult(t("voiceRecordingTitle"), t("voiceRecordingBody")(seconds));
  try {
    const response = await fetch(gatewayHttpUrl("/api/voice/transcribe"), {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ seconds }),
    });
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(payload.error || "Voice transcription failed");
    }
    setVoiceResult(
      payload.result.text,
      `sample rate ${payload.result.sample_rate}, channels ${payload.result.channels}`,
    );
    logLine("Voice", JSON.stringify(payload.result, null, 2));
  } catch (error) {
    setVoiceResult(t("voiceErrorTitle"), String(error));
    logLine("Voice", String(error), "error");
  }
}

els.connectBtn.addEventListener("click", connectSocket);
els.disconnectBtn.addEventListener("click", () => disconnectSocket("manual"));
els.sendTask.addEventListener("click", sendTask);
els.clearLog.addEventListener("click", () => {
  els.log.innerHTML = "";
});
els.taskInput.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
    sendTask();
  }
});
els.visionToggle.addEventListener("change", setVisionBadge);
els.themeToggle.addEventListener("click", () => {
  setTheme(state.theme === "dark" ? "light" : "dark");
});
els.languageToggle.addEventListener("click", () => {
  setLanguage(state.language === "zh" ? "en" : "zh");
});
els.refreshChannels.addEventListener("click", loadChannels);
els.refreshDevices.addEventListener("click", loadVoiceDevices);
els.voiceRecord.addEventListener("click", transcribeVoice);
els.navItems.forEach((item) => {
  item.addEventListener("click", () => {
    els.navItems.forEach((node) => node.classList.remove("is-active"));
    item.classList.add("is-active");
    const target = document.getElementById(item.dataset.section);
    if (target) {
      target.scrollIntoView({ behavior: "smooth", block: "start" });
    }
  });
});

setTheme(state.theme);
setLanguage(state.language);
setScreenshot("");
setResultCard(t("noResult"), t("resultSeed"));
setVoiceResult(t("noVoice"), t("voice_empty_body"));
loadChannels();
loadVoiceDevices();
logLine("Boot", t("boot"));
