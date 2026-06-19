# EyeForge v2.0 Roadmap

## Goal

EyeForge v2.0 focuses on a unified local service layer, command-driven interaction, multi-channel access, AI group collaboration, and runtime/loader extensibility. The guiding rule is: keep desktop, Web UI, HAPI, iLink, and AI group behavior consistent before adding deeper platform features.

## Milestone 1: Service and Command Foundation

Status: in progress.

- Keep the built-in Rust gateway and HAPI server as separate services with explicit start/stop behavior.
- Add a shared backslash command parser for desktop, WebSocket, iLink, and future group channels.
- Support `\help`, `\new`, `\view on/off`, `\ai group on/off`, and `\ai group start/stop`.
- Stream runtime step events to channels so remote users can see "running", each action, and final result.
- Align Rust and HAPI AI prompts so adaptive execution can choose batch actions when the next steps are certain, or one action when feedback is needed.

## Milestone 2: Modern UI and Mode Controls

Status: planned.

- Add separate controls for gateway/Web UI availability and desktop task execution mode.
- Preserve shared config so desktop and Web UI do not drift.
- Modernize the visual language with acrylic/glass panels, purposeful gradients, hover hints, and clearer running-state feedback.
- Keep Chinese as a first-class UI language, including footer labels, channel names, and QR/login instructions.

## Milestone 3: AI Group Collaboration

Status: planned.

- Refactor AI groups into explicit local server/client sessions.
- Support LAN by binding IPv4/IPv6 hosts and ports from config.
- Support WAN through public bind plus token authentication first; relay/tunnel/NAT traversal can be a later feature.
- Route command messages through the same command parser before LLM dispatch.
- Keep Codex/OpenCode/HAPI dispatch compatible with the shared EyeForge action-plan schema.

## Milestone 4: AI Loader and Tool API

Status: planned.

- Integrate `C:\Users\ghr16\Desktop\AIloaderr` first as an external helper/subprocess, then decide whether to merge its Rust modules.
- Add runtime detection, download/install progress, install wizard, and manual install path selection.
- Support standalone `swigopenvino`, Python `openvino`, standalone `openvino-genai`, and Python `openvino-genai` variants.
- Add OpenAI Tools-compatible schema and endpoints while keeping legacy EyeForge endpoints during the transition.

## Current Acceptance Checks

- Desktop task input handles backslash commands without sending them to the LLM.
- WebSocket task execution streams runtime status events before the final result.
- HAPI and Rust built-in Codex/OpenCode prompts use the same adaptive batch/step policy.
- Command effects that change config persist immediately and restart or stop affected services.
