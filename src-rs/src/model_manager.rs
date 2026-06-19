use iced::widget::{
    button, container, horizontal_rule, row, scrollable, text, text_input, Column, Row,
};
use iced::{Color, Element, Fill, Task};

#[derive(Debug, Clone)]
pub enum ModelManagerMessage {
    ScanPathChanged(String),
    Scan,
    RefreshBackends,
    BackendSelected(String),
    BackendPathChanged(String),
    SetBackendPath,
}

#[derive(Debug, Clone)]
pub struct ModelEntry {
    pub format: String,
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct ModelManagerState {
    pub scan_path: String,
    pub scan_results: Vec<ModelEntry>,
    pub scanning: bool,
    pub status: String,
    pub backends_available: Vec<BackendEntry>,
    pub backends_missing: Vec<BackendEntry>,
    pub kb_count: usize,
    pub selected_backend_id: String,
    pub manual_backend_path: String,
    pub editing_path: String,
}

#[derive(Debug, Clone)]
pub struct BackendEntry {
    pub name: &'static str,
    pub description: &'static str,
}

impl ModelManagerState {
    pub fn new() -> Self {
        let statuses = ailo::backend::BackendInfo::detect_all();
        let mut available = Vec::new();
        let mut missing = Vec::new();
        for s in &statuses {
            let entry = BackendEntry {
                name: s.info.name,
                description: s.info.description,
            };
            if s.available {
                available.push(entry);
            } else {
                missing.push(entry);
            }
        }
        Self {
            scan_path: String::new(),
            scan_results: Vec::new(),
            scanning: false,
            status: String::new(),
            backends_available: available,
            backends_missing: missing,
            kb_count: ailo::kb::KnowledgeBase::list_all()
                .map(|v| v.len())
                .unwrap_or(0),
            selected_backend_id: String::new(),
            manual_backend_path: String::new(),
            editing_path: String::new(),
        }
    }

    pub fn refresh_backends(&mut self) {
        let statuses = ailo::backend::BackendInfo::detect_all();
        self.backends_available.clear();
        self.backends_missing.clear();
        for s in &statuses {
            let entry = BackendEntry {
                name: s.info.name,
                description: s.info.description,
            };
            if s.available {
                self.backends_available.push(entry);
            } else {
                self.backends_missing.push(entry);
            }
        }
        self.kb_count = ailo::kb::KnowledgeBase::list_all()
            .map(|v| v.len())
            .unwrap_or(0);
    }

    pub fn scan_models(&mut self) {
        let path = std::path::Path::new(&self.scan_path);
        if !path.exists() {
            self.status = format!("路径不存在: {}", self.scan_path);
            return;
        }
        self.scanning = true;
        self.status = String::new();
        self.scan_results.clear();

        match ailo::format::scan_for_models(path) {
            Ok(models) => {
                for (fmt, p) in models {
                    let size = std::fs::metadata(&p).ok().map(|m| m.len()).unwrap_or(0);
                    self.scan_results.push(ModelEntry {
                        format: fmt.to_string(),
                        path: p.to_string_lossy().to_string(),
                        size,
                    });
                }
                self.status = format!("扫描完成，找到 {} 个模型", self.scan_results.len());
            }
            Err(e) => {
                self.status = format!("扫描失败: {}", e);
            }
        }
        self.scanning = false;
    }

    pub fn update(&mut self, msg: ModelManagerMessage) -> Task<ModelManagerMessage> {
        match msg {
            ModelManagerMessage::ScanPathChanged(v) => {
                self.scan_path = v;
            }
            ModelManagerMessage::Scan => {
                self.scan_models();
            }
            ModelManagerMessage::RefreshBackends => {
                self.refresh_backends();
            }
            ModelManagerMessage::BackendSelected(v) => {
                self.selected_backend_id = v;
            }
            ModelManagerMessage::BackendPathChanged(v) => {
                self.editing_path = v;
            }
            ModelManagerMessage::SetBackendPath => {
                self.manual_backend_path = self.editing_path.clone();
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, ModelManagerMessage> {
        let accent = Color::from_rgb(0.4, 0.6, 1.0);
        let green = Color::from_rgb(0.3, 0.8, 0.4);
        let red = Color::from_rgb(0.9, 0.3, 0.3);

        let mut col = Column::new().spacing(12).padding(20);

        // Title
        col = col.push(
            text("AI 模型管理")
                .size(16)
                .color(Color::from_rgb(0.85, 0.85, 0.85)),
        );
        col = col.push(horizontal_rule(1));

        // Backends section
        col = col.push(
            Row::new()
                .push(text("推理后端").size(14).color(accent))
                .push(button("刷新").on_press(ModelManagerMessage::RefreshBackends))
                .spacing(12)
                .align_y(iced::Alignment::Center),
        );
        for b in &self.backends_available {
            col = col.push(
                row![
                    text(format!("  ✓ {}", b.name)).size(13).color(green),
                    text(b.description)
                        .size(12)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                ]
                .spacing(8),
            );
        }
        for b in &self.backends_missing {
            col = col.push(
                row![
                    text(format!("  ✗ {}", b.name)).size(13).color(red),
                    text(b.description)
                        .size(12)
                        .color(Color::from_rgb(0.55, 0.55, 0.55)),
                ]
                .spacing(8),
            );
        }
        if self.backends_available.is_empty() && self.backends_missing.is_empty() {
            col = col.push(
                text("  未检测到后端信息")
                    .size(12)
                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }

        col = col.push(horizontal_rule(1));
        col = col.push(text("选择推理后端").size(14).color(accent));
        let backend_ids = [
            "auto", "llama", "openvino", "onnx", "pytorch", "candle", "tensorrt", "mnn",
        ];
        let display_names = [
            "自动检测",
            "llama.cpp (GGUF)",
            "OpenVINO (IR)",
            "ONNX Runtime",
            "PyTorch",
            "Candle (Rust)",
            "TensorRT",
            "MNN",
        ];
        for (id, name) in backend_ids.iter().zip(display_names.iter()) {
            let selected = self.selected_backend_id == *id
                || (self.selected_backend_id.is_empty() && *id == "auto");
            let prefix = if selected { "●" } else { "○" };
            col = col.push(
                button(row![
                    text(format!(" {} ", prefix)).size(13),
                    text(*name).size(13)
                ])
                .on_press(ModelManagerMessage::BackendSelected(id.to_string()))
                .style(if selected {
                    button::primary
                } else {
                    button::text
                }),
            );
        }

        col = col.push(horizontal_rule(1));
        col = col.push(text("后端路径（手动指定）").size(14).color(accent));
        let current_path = if self.manual_backend_path.is_empty() {
            "PATH 自动查找"
        } else {
            &self.manual_backend_path
        };
        col = col.push(
            text(format!("当前: {}", current_path))
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
        );
        col = col.push(
            row![
                text_input("输入后端可执行文件路径...", &self.editing_path)
                    .on_input(ModelManagerMessage::BackendPathChanged),
                button("设置路径").on_press(ModelManagerMessage::SetBackendPath),
            ]
            .spacing(8),
        );

        col = col.push(horizontal_rule(1));

        // KB section
        col = col.push(text("知识库 (RAG)").size(14).color(accent));
        col = col.push(
            text(format!("  已创建知识库: {}", self.kb_count))
                .size(13)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
        );

        col = col.push(horizontal_rule(1));

        // Scan section
        col = col.push(text("扫描模型").size(14).color(accent));
        col = col.push(
            row![
                text_input("输入目录路径...", &self.scan_path)
                    .on_input(ModelManagerMessage::ScanPathChanged)
                    .width(Fill),
                button("扫描").on_press(ModelManagerMessage::Scan),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );

        if !self.status.is_empty() {
            col = col.push(
                text(&self.status)
                    .size(13)
                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }

        // Results
        if !self.scan_results.is_empty() {
            col = col.push(horizontal_rule(1));
            col = col.push(text("扫描结果").size(14).color(accent));
            for m in &self.scan_results {
                let size_str = if m.size > 1024 * 1024 * 1024 {
                    format!("{:.1} GB", m.size as f64 / (1024.0 * 1024.0 * 1024.0))
                } else if m.size > 1024 * 1024 {
                    format!("{:.1} MB", m.size as f64 / (1024.0 * 1024.0))
                } else if m.size > 1024 {
                    format!("{:.1} KB", m.size as f64 / 1024.0)
                } else {
                    format!("{} B", m.size)
                };
                col = col.push(
                    row![
                        text(format!("  [{}]", m.format)).size(12).color(accent),
                        text(&m.path).size(12).color(Color::from_rgb(0.6, 0.6, 0.6)),
                        text(size_str)
                            .size(11)
                            .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    ]
                    .spacing(8),
                );
            }
        }

        container(scrollable(col)).width(Fill).height(Fill).into()
    }
}
