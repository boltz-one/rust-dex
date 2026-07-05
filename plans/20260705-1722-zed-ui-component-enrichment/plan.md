---
title: "Zed UI Component Enrichment for crates/ui"
description: "Làm giàu crates/ui bằng ý tưởng/port có chọn lọc từ Zed: chrome, syntax highlighting, terminal PTY"
status: pending
priority: P2
effort: 3-4w (A) + 2-3w (B) + open-ended (C, high risk)
branch: main
tags: [ui, gpui, zed-port, editor, terminal, research]
created: 2026-07-05
---

# Zed UI Component Enrichment

**LOẠI PLAN: nghiên cứu/thiết kế, CHƯA implement gì.** Đọc hết plan này + 3 phase file trước khi bắt đầu code. Mỗi phase độc lập triển khai được — bạn tự chọn điểm dừng (chỉ A, A+B, hoặc cả A+B+C) sau khi cân nhắc Risk Assessment và Unresolved Questions ở cuối file này.

## Bối cảnh

`others/zed/` là vendor source Zed (gitignored, read-only, không phải workspace member) dùng để tham khảo/port. Base hiện có: `code_editor.rs` (106 dòng, TextInput multiline + gutter tĩnh, KHÔNG highlight thật), `command_palette/` (279+dòng, tự viết fuzzy picker đơn giản — KHÔNG phải port từ Zed's `picker` crate), `tab.rs`/`tab_bar.rs`/`resizable.rs`/`sidebar.rs` (chrome cơ bản, không có dock/pane-group hệ thống).

**Phát hiện quan trọng khi kiểm tra `others/zed/crates/*/Cargo.toml`** (không có trong 2 báo cáo nghiên cứu gốc): `picker`, `tab_switcher`, `breadcrumbs`, `title_bar` — dù LOC thấp — đều khai báo dependency trực tiếp vào `workspace`, `editor`, `project`, `db`, `theme_settings`, `zed_actions`, `menu`, `ui_input` trong `Cargo.toml`. Nghĩa là "port" các crate này không thể là copy-paste-compile; phải **rewrite dựa trên ý tưởng kiến trúc**, tái dùng primitive sẵn có của base (`TextInput`, `List`, `Modal`, `ListItem`). Điều này củng cố quyết định giữ `command_palette` hiện tại làm nền, không port thẳng `picker` crate của Zed.

## Phases

| Phase | Tên | Rủi ro | Dependency mới | Status |
|---|---|---|---|---|
| A | [Visual chrome components](./phase-01-visual-chrome-components.md) | Thấp | Không | Completed (2026-07-05) |
| B | [Real syntax highlighting + terminal chrome](./phase-02-real-syntax-highlighting-and-terminal-chrome.md) | Trung bình | `rope`, `tree-sitter` + 5 grammar (Rust/JS/TS/Markdown/JSON) | Completed (2026-07-05) |
| C | [Real terminal PTY + text buffer](./phase-03-real-terminal-pty-and-text-buffer.md) | Cao | `alacritty_terminal`, `vte`, `async-channel` | **Completed cho macOS/Unix (2026-07-05). Linux/Windows CHƯA verify.** |

**Cập nhật (vòng 2, cùng ngày)**: user xác nhận qua `/goal` chấp nhận rủi ro dependency (giải quyết Unresolved Question #2) và yêu cầu hoàn thiện Phase C. Đã implement `crates/terminal` (PTY thật qua `alacritty_terminal` 0.25.1 — bản publish trên crates.io, không phải fork riêng của Zed) + `crates/ui/src/components/terminal_view.rs` (real shell, real I/O, verified bằng test thật + chạy `ui_gallery` thật, xác nhận qua `ps` không leak process). **Giới hạn còn lại sau vòng 2, trung thực**: chỉ verify được macOS/Unix (môi trường phiên này không có Linux/Windows). Xem phase-03 § "Ghi chú triển khai thực tế" và § Success Criteria để biết chi tiết.

**Cập nhật (vòng 3, cùng ngày)**: hoàn thiện các mục tồn đọng của Phase C — resize thật nối với kích thước pane đo được (không còn cố định 24x80), màu ANSI per-cell thật (16 màu chuẩn + bảng 256 màu, render qua `StyledText` giống `code_editor.rs`), mở rộng key encoding (F1-F12, Home/End/PageUp/PageDown/Delete/Insert). Phát hiện + sửa 1 bug thật khi viết test: PTY echo dòng lệnh gõ vào (chưa tô màu) xuất hiện trước output thật trong grid, ban đầu làm test match nhầm ký tự. 6/6 test crate `terminal` pass (gồm 1 test resize thật + 1 test màu ANSI thật end-to-end qua shell thật). Giới hạn còn lại: vẫn chỉ verify macOS/Unix; chưa có mouse/hyperlink/scrollback UI/vi-mode (ngoài phạm vi, xem `terminal_view.rs`'s doc).

Phase A + B + C(macOS) là trạng thái hiện tại — không phải "Done" tuyệt đối theo đúng chữ của tiêu chí gốc phase-03 (thiếu multi-platform), nhưng là kết quả trung thực nhất có thể trong môi trường 1 hệ điều hành.

Mỗi phase file có: Context links, Overview, Key Insights (trích LOC/coupling từ 2 báo cáo + phát hiện Cargo.toml ở trên), Requirements, Architecture (tên crate mới cụ thể), ADR Rationale, Related code files (base + `others/zed/crates/...`), Implementation Steps, Todo list, Success Criteria, Risk Assessment, Security Considerations, Next steps.

## Ngoài phạm vi (mọi phase)

Full LSP, workspace persistence/DB (dùng `crates/db` riêng theo `docs/system-architecture.md` § Add Persistence khi cần), extension host, AI/agent panel, vim mode, collab/remote dev, `project_panel` (20kLOC, cần Project/Worktree/Git model), Zed's `sidebar` (23kLOC, agent-specific), full `workspace` system (48kLOC).

## Unresolved Questions

1. ~~Điểm dừng phase~~ — **Đã giải quyết**: user chỉ định làm cả A+B+C qua `/goal`.
2. ~~Chấp nhận dependency nặng?~~ — **Đã giải quyết**: user chấp nhận qua `/goal`. Binary size đo được thật: +3.53MiB cho 4 grammar thêm (xem phase-02); `alacritty_terminal`/`vte`/`async-channel` đã thêm cho Phase C.
3. ~~Nâng cấp `command_palette` lên Picker/delegate?~~ — **Đã giải quyết**: KHÔNG generic hóa (2 use-case vẫn khác hình dạng hành vi), chỉ trích phần chrome trùng lặp thuần túy thành `overlay_chrome.rs` — xem phase-01 § ADR mục 1 (cập nhật).
4. Base có định hỗ trợ split-pane/dock layout thật (nhiều editor tab đồng thời) hay pane-group ở Phase A chỉ là chrome trình diễn? — **Vẫn mở**, chưa cần quyết định vì chưa có nhu cầu cụ thể.
5. ~~`crates/syntax_theme` có khớp tree-sitter convention?~~ — **Đã giải quyết + tìm ra bug thật**: convention khớp, nhưng `style_for_name` KHÔNG tự fallback theo dotted-prefix như tưởng — đã phát hiện + sửa (xem phase-02 § test coverage).
6. **Mới**: Phase C's grid resize CHƯA nối với `PaneGroup` thật (cố định 24x80) — cần làm nếu muốn terminal thực dụng trong layout thật.
7. **Mới**: Phase C CHƯA verify Linux/Windows — cần môi trường đa nền tảng để hoàn tất đúng nghĩa tiêu chí gốc của phase-03.
