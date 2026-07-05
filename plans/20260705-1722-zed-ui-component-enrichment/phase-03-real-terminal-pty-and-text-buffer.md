# Phase C — Real Terminal PTY + Text Buffer Integration

## Context links

- Plan overview: [plan.md](./plan.md)
- Research: [researcher-02-editor-terminal-stack.md](./research/researcher-02-editor-terminal-stack.md)
- Previous phase: [phase-02](./phase-02-real-syntax-highlighting-and-terminal-chrome.md)

## Overview

- Date: 2026-07-05
- Description: Port `terminal` crate (alacritty_terminal-based PTY I/O) thành PTY thật, bridge qua `gpui_platform` facade để giữ nguyên convention "không `#[cfg(target_os)]` ngoài platform crates". `terminal_view` chỉ dùng làm tài liệu tham khảo — viết lại hoàn toàn vì coupling GPUI element quá chặt với Zed's Workspace/Pane. Đây là phase rủi ro cao nhất trong toàn bộ plan.
- Priority: P3 (chỉ làm nếu Phase A+B đã ổn định và user chấp nhận rủi ro cross-platform)
- Implementation status: **Completed cho macOS/Unix (2026-07-05). Linux/Windows CHƯA được verify — xem "Ghi chú triển khai thực tế".**
- Review status: Not Reviewed

**Ghi chú triển khai thực tế:**
- Không port literal 9007 dòng của Zed's `terminal` crate (đúng như ADR gốc dự đoán — crate đó gắn chặt với `settings`/`task`/`theme_settings`/`release_channel` của Zed, base không có các crate này). Thay vào đó viết `crates/terminal` mới, mỏng, dựa theo Ý TƯỞNG kiến trúc từ `others/zed/crates/terminal/src/alacritty.rs` (hàm `open_pty`/`new_term`/`spawn_event_loop`/`resize`) nhưng code thực tế viết lại từ đầu, gọi thẳng bản `alacritty_terminal` publish trên crates.io (`0.25.1`) — Zed dùng bản fork riêng (`git = "zed-industries/alacritty"`) nên API không đảm bảo giống hệt; đã verify từng API thật qua `cargo check` (không đoán).
- **PTY syscall KHÔNG cần đi qua `gpui_platform` facade như ADR gốc dự đoán** — phát hiện quan trọng: `alacritty_terminal::tty::new()` tự đóng gói toàn bộ forkpty(Unix)/ConPTY(Windows) NỘI BỘ, không lộ `#[cfg(target_os)]` nào ra caller. `crates/terminal` gọi thẳng `tty::new(...)` mà không cần viết bất kỳ `#[cfg(target_os)]` nào — nguyên tắc "no platform gate ngoài whitelist" được thoả mãn tự nhiên vì branching nằm hoàn toàn trong dependency, không phải code base tự viết (giống cách `wgpu`/`cosmic-text` đã được dùng tự do mà không cần qua facade).
- Kiến trúc thực tế: `crates/terminal` (không phụ thuộc `gpui`) — `Terminal::spawn()` trả về `(Terminal, async_channel::Receiver<Event>)`; `Terminal::write_input`/`resize`/`screen_lines`/`shutdown`. `Drop` tự gọi `shutdown()` (an toàn vì `Terminal` không `Clone`, chỉ 1 owner) — giảm thiểu đúng "Resource leak risk" mà Risk Assessment gốc lo ngại.
- `crates/ui/src/components/terminal_view.rs` (mới) — `TerminalView` bridge PTY event → `cx.notify()` qua `cx.spawn(async move |this, cx| while events.recv().await.is_ok() {...})` (pattern đã có sẵn trong `toast_stack.rs`, không phát minh pattern mới). Key encoding tối giản: in được + Enter/Backspace/Tab/Escape/mũi tên + Ctrl+letter — CHƯA có Option-as-Meta (macOS), function keys, bracketed-paste.
- **Giới hạn thực tế đã ghi rõ trong code** (không phải "hoàn chỉnh"): (1) render plain text đơn sắc, KHÔNG có màu ANSI per-cell thật (`screen_lines()` bỏ qua color/attribute của grid cố ý); (2) grid size cố định 24x80, CHƯA nối với resize thật của `PaneGroup`; (3) không mouse, không hyperlink, không vi-mode/scrollback UI.
- **Verify thật, không suy đoán**: 1 test trong `crates/terminal` spawn `/bin/sh` thật, gửi `echo <marker>\n` qua `write_input`, đọc lại `screen_lines()` và assert thấy marker — PASS. Chạy `examples/ui_gallery` thật (Layout page): xác nhận qua `ps` rằng app con thật sự spawn ra 1 process `/bin/zsh` con; sau khi đóng app, xác nhận process con cũng bị dọn sạch (không leak) — khớp với thiết kế `Drop`.
- **KHÔNG verify được Linux/Windows** — môi trường phiên này chỉ có macOS. Theo đúng tiêu chí gốc của phase này ("không merge nếu chỉ test 1 OS"), phase này KHÔNG được coi là "Done" đầy đủ cho toàn bộ Success Criteria — chỉ macOS/Unix path được xác nhận hoạt động.
- `make fmt-check`/`cargo check --all-targets` scoped cho các crate mình đổi (`boltz-ui`, `boltz-rope`, `boltz-language-core`, `boltz-terminal`, `ui_gallery`, `boltz-app`) pass sạch; 77/77 test pass.

## Key Insights (từ nghiên cứu 02)

| Component | LOC | External deps | Zed coupling |
|---|---|---|---|
| `terminal` | 9,007 | `alacritty_terminal`, `vte`, `async-channel`, `futures`, `libc` (Unix), `windows` (Windows) | Trung bình — `gpui`, `collections`, `theme`, `settings`, `task` |
| `terminal_view` | 9,635 | — (view-layer) | Cao — 100% GPUI element/view semantics |

- **`#[cfg(target_os)]` thực tế trong `others/zed/crates/terminal/src`**: xác nhận qua grep trực tiếp — tập trung ở `pty_info.rs` (Windows-only path), `terminal.rs` (nhiều block `cfg(not(target_os = "windows"))` và `cfg(any(target_os = "linux", target_os = "freebsd"))`, dòng 1628/1642/1661/2505/3398/3460/3532/3755/3824/3940/3942), `alacritty.rs` (Windows ConPTY path), `mappings/keys.rs` (macOS Option-as-Meta), `alacritty/hyperlinks.rs` (Windows vs Unix URL parsing). Số lượng khối thực tế nhiều hơn con số "6" mà báo cáo 02 nêu (báo cáo đếm theo "instance nhóm", không phải grep dòng-đơn) — nhưng tất cả đều nằm gọn trong PTY acquisition/platform-quirk logic, không rải rác khắp file.
- **`terminal/Cargo.toml`** xác nhận: `[target.'cfg(windows)'.dependencies] windows.workspace = true` — Windows PTY dùng crate `windows` riêng, không chung code path với Unix `libc`/`forkpty`.
- alacritty_terminal đã đóng gói ~95% platform logic (forkpty Unix / ConPTY Windows) theo báo cáo 02 — nhưng phần `#[cfg]` còn lại (key mapping macOS Option-as-Meta, hyperlink parsing, resize signaling) phải tự xử lý.
- **Cross-platform gotcha cụ thể** (báo cáo 02): Unix cần `forkpty()` + fd management qua async-channel↔GPUI event loop bridge; Windows cần ConPTY handle + legacy console mode detect; cả 2 cần SIGWINCH/resize signaling đồng bộ viewport GPUI.
- `terminal_view` không tái sử dụng được — phải viết lại hoàn toàn dựa trên GPUI element base của base project, chỉ dùng làm tài liệu đọc hiểu layout/scroll/selection UX.

## Requirements

1. `crates/terminal` mới: port `alacritty_terminal` + `vte` integration logic từ `others/zed/crates/terminal/src/terminal.rs`, loại bỏ toàn bộ `gpui`-specific coupling ở tầng này (giữ core: PTY spawn, VT100 parse, screen buffer state) — KHÔNG port phần đã dùng `theme`/`settings` của Zed trực tiếp, thay bằng interface tối giản base tự định nghĩa.
2. PTY syscall (forkpty Unix / ConPTY Windows) PHẢI đi qua `gpui_platform` facade — không được có `#[cfg(target_os)]` trong `crates/terminal` (vi phạm `docs/code-standards.md`: platform gate chỉ ở `gpui_platform`/`gpui_macos`/`gpui_linux`/`gpui_windows`/`font_kit`).
3. `TerminalView` mới trong `crates/ui` (không port `terminal_view`): render buffer từ `crates/terminal`, dùng element model của base (tương tự cách `code_editor.rs` render text hiện tại), cắm vào `PaneGroup`/`TerminalPanel` chrome đã có ở Phase B.
4. Resize signaling: khi panel co giãn (qua `ResizablePanelGroup` đã có ở Phase A), gọi `TIOCSWINSZ`(Unix)/`ResizePseudoConsole`(Windows) qua facade — không phải gọi trực tiếp trong `crates/ui`/`crates/terminal`.
5. Async I/O bridge: đọc PTY output (async-channel) → đẩy vào GPUI's `cx.spawn`/executor mà không block main thread — audit `crates/gpui/src/executor` hoặc pattern `cx.background_executor()` đã dùng ở `gpui_platform.rs`.

## Architecture

```
crates/gpui_platform/          (SỬA — thêm facade functions)
└── src/gpui_platform.rs       + pub fn pty_open() -> Result<PtyHandle>
                                + pub fn pty_resize(handle: &PtyHandle, rows, cols)
                                (impl thật nằm ở gpui_macos/gpui_linux/gpui_windows,
                                 hoặc dùng alacritty_terminal's cross-platform Pty type
                                 trực tiếp trong facade nếu nó đã tự đóng gói #[cfg] nội bộ —
                                 cần xác nhận trước khi quyết định impl location, xem ADR mục 3)

crates/terminal/               (MỚI, package="boltz-terminal" nếu publish)
├── src/terminal.rs            core: PTY session state, VT100 parse (vte), screen buffer
│                               deps: alacritty_terminal, vte, async-channel, futures, anyhow
└── Cargo.toml                 KHÔNG chứa #[cfg(target_os)] — mọi OS-branch qua gpui_platform

crates/ui/src/components/
└── terminal_view.rs           (MỚI) render crates/terminal's buffer, thay TerminalPanel
                                chrome tĩnh của Phase B bằng nội dung PTY thật
```

Dependency graph:
```
crates/terminal → gpui_platform (facade PTY calls, KHÔNG gọi libc/windows trực tiếp)
                → alacritty_terminal, vte, async-channel (external, cross-platform)
crates/ui → crates/terminal (render buffer)
gpui_platform → gpui_macos | gpui_linux | gpui_windows (PTY impl thật, nếu facade không đủ)
```

## ADR Rationale

**1. Vì sao PTY syscall phải qua `gpui_platform` facade, không nằm thẳng trong `crates/terminal`?**
- Context: `docs/code-standards.md` liệt kê rõ: platform gate CHỈ được phép ở `gpui_platform`/`gpui_macos`/`gpui_linux`/`gpui_windows`/`font_kit`. `crates/terminal` không nằm trong whitelist này.
- Decision: `crates/terminal` gọi `gpui_platform::pty_open()`/`pty_resize()` — không tự viết `#[cfg(target_os)]`.
- Why this over alternatives: Vi phạm convention làm hỏng nguyên tắc "app code never branches" đã áp dụng nhất quán từ `crates/app/src/main.rs` đến `gpui_platform.rs` hiện tại (đã có tiền lệ `set_application_icon_png` dùng đúng pattern này). Giữ nhất quán giúp review dễ, tránh rải `#[cfg]` từ Terminal lan sang các crate khác trong tương lai.

**2. Vì sao `terminal_view` KHÔNG port mà viết lại?**
- Context: Báo cáo 02 xác nhận `terminal_view` (9.6kLOC) "100% GPUI-specific — any port requires full GPUI element reimplementation", coupling CAO với Zed's Workspace/Pane/Item semantics.
- Decision: Chỉ đọc `terminal_view` để hiểu UX (scroll-back buffer render, selection/copy, cursor blink, alternate-screen handling) — code thực viết mới dựa trên base's element patterns (giống cách `code_editor.rs`/`TextInput` đã tự viết render logic, không port từ `editor` crate của Zed).
- Why: Base's element/view API (Phase A/B đã thiết lập pattern qua `code_editor.rs`, `PaneGroup`) khác đủ nhiều so với Zed's `Pane`/`Item` trait system khiến port cơ giới tốn effort ngang với viết lại, trong khi viết lại cho phép tái dùng đúng những gì Phase A/B đã xây (TabBar, PaneGroup) thay vì 2 hệ thống chrome song song.

**3. Impl PTY thật nằm ở `gpui_platform` trực tiếp hay ở `gpui_macos`/`gpui_linux`/`gpui_windows`?**
- Context: `alacritty_terminal` đã tự đóng gói ~95% khác biệt OS (theo báo cáo 02) — nghĩa là bản thân `alacritty_terminal`'s `Pty` type có thể đã cross-platform mà không cần base tự viết `#[cfg]` thêm.
- Decision CHƯA CHỐT (cần xác minh code `alacritty_terminal` thật trước khi implement) — đề xuất 2 phương án:
  - (a) Nếu `alacritty_terminal::tty::Pty::new()` đã nhận `WindowSize`/`Options` và tự route OS nội bộ mà không lộ `#[cfg]` ra caller: gọi thẳng trong `gpui_platform.rs` (giống `current_platform()` đã làm), không cần thêm code ở `gpui_macos`/`gpui_linux`/`gpui_windows`.
  - (b) Nếu vẫn cần các cuộc gọi `TIOCSWINSZ`/`ioctl` thủ công (báo cáo 02 liệt kê rõ đây là điểm cần base tự xử lý) và cần logic khác biệt thật (macOS Option-as-Meta ở `mappings/keys.rs`): đặt trong `gpui_macos`/`gpui_linux`/`gpui_windows` như các platform-specific fn khác (theo pattern `MetalHeadlessRenderer`/`MacPlatform` hiện có), expose qua trait mới trong `crates/gpui/src/platform.rs` (theo `system-architecture.md` § "Add Platform-Specific Behavior": "Add method to platform.rs trait, implement in gpui_macos/gpui_linux/gpui_windows, call via cx.platform()").
- Why: Tuân đúng extensibility point đã document sẵn trong `docs/system-architecture.md`, không tạo pattern mới song song.

## Related code files

**Base (đọc/sửa):**
- `crates/gpui_platform/src/gpui_platform.rs` — thêm facade fn, tham khảo cách `current_platform()`/`set_application_icon_png` đã cfg-gate
- `crates/gpui/src/platform.rs` — nếu cần thêm method vào `Platform` trait (theo ADR mục 3 phương án b)
- `crates/gpui_macos/src/platform.rs`, `crates/gpui_linux/src/linux/platform.rs`, `crates/gpui_windows/src/platform.rs` — nếu cần impl PTY riêng theo OS
- `crates/ui/src/components/terminal_panel.rs` (từ Phase B) — thay chrome tĩnh bằng buffer thật

**Zed vendor (tham khảo/port có chọn lọc):**
- `others/zed/crates/terminal/src/terminal.rs` — core PTY+VT100 logic, port có chọn lọc (bỏ `theme`/`settings` Zed-specific)
- `others/zed/crates/terminal/src/pty_info.rs` — Windows-specific PTY info, tham khảo cho `gpui_windows` impl
- `others/zed/crates/terminal/src/alacritty.rs`, `alacritty/hyperlinks.rs` — Windows ConPTY path + hyperlink OS-branch
- `others/zed/crates/terminal/src/mappings/keys.rs` — macOS Option-as-Meta key mapping
- `others/zed/crates/terminal/Cargo.toml` — đối chiếu dependency list (`alacritty_terminal`, `vte`, `async-channel`, `futures-lite`, `sysinfo`, `windows` cho Windows target)
- `others/zed/crates/terminal_view/` — CHỈ ĐỌC để hiểu UX, không port code

## Implementation Steps

1. Đọc source `alacritty_terminal` (crates.io hoặc vendor version Zed dùng) để xác định phương án (a) hay (b) ở ADR mục 3 — đây là bước xác minh bắt buộc TRƯỚC KHI viết code, không giả định.
2. Thêm `alacritty_terminal`, `vte`, `async-channel`, `futures`, `futures-lite` vào `[workspace.dependencies]` (kiểm tra version Zed đang pin — Unresolved Question #3 ở báo cáo 02).
3. Tạo `crates/terminal/` với core PTY session struct — spawn/read/write/resize — gọi qua facade đã quyết ở bước 1.
4. Nếu phương án (b): thêm method vào `crates/gpui/src/platform.rs` trait, impl ở 3 backend crate, route qua `gpui_platform`.
5. Bridge async I/O: dùng `cx.background_executor()` (đã có sẵn trong `gpui_platform.rs`) để đọc PTY output không block main thread, gửi qua channel vào GPUI's update loop.
6. Viết `terminal_view.rs` trong `crates/ui`: render screen buffer (dùng `Rope`/text rendering pattern đã có từ Phase B nếu áp dụng được, hoặc grid-cell renderer riêng cho VT100 screen model — 2 mô hình dữ liệu khác nhau, không ép dùng chung).
7. Resize signaling: hook vào `ResizablePanelGroup`'s `on_drag_move` callback (đã có ở Phase A) để gọi `pty_resize()`.
8. Test thủ công trên cả 3 platform trước khi coi Success Criteria đạt — không có CI cross-platPTY test tự động sẵn có (base's CI hiện chỉ `cargo check`/`fmt`/`clippy`, không có runtime PTY spawn test).
9. `make fmt-check && make check-all`.

## Todo list

- [ ] Xác minh alacritty_terminal's Pty API thật (ADR mục 3) trước khi code
- [ ] Pin version `alacritty_terminal`/`vte`/`async-channel` (đối chiếu Zed's lockfile)
- [ ] `crates/terminal/` core PTY+VT100, KHÔNG chứa `#[cfg(target_os)]`
- [ ] Facade fn trong `gpui_platform` (+ backend impl nếu cần)
- [ ] Async I/O bridge qua `cx.background_executor()`
- [ ] `terminal_view.rs` render buffer thật
- [ ] Resize signaling nối với `ResizablePanelGroup`
- [ ] Test thủ công macOS + Linux + Windows (ghi lại kết quả từng platform)
- [ ] `make fmt-check && make check-all` pass

## Success Criteria

- [x] Không một dòng `#[cfg(target_os)]` nào xuất hiện ngoài `gpui_platform`/`gpui_macos`/`gpui_linux`/`gpui_windows` — xác nhận: `crates/terminal` không viết `#[cfg(target_os)]` nào (branching nằm trong `alacritty_terminal`, xem ghi chú triển khai).
- [x] (macOS/Unix only) Terminal panel spawn được shell thật (`$SHELL` trên Unix), nhận input, hiển thị output không crash — verified qua test thật + chạy `ui_gallery` thật.
- [ ] (Windows) `cmd.exe`/PowerShell — **CHƯA verify**, không có máy Windows trong phiên này.
- [ ] Resize theo panel — **CHƯA nối dây thật**, `Terminal::resize()` tồn tại và hoạt động độc lập nhưng `TerminalView` hiện dùng grid cố định 24x80, chưa gọi resize khi `PaneGroup` co giãn.
- [x] Đóng panel/app không leak PTY process — verified bằng `ps` thật trước/sau khi đóng `ui_gallery`: process `/bin/zsh` con biến mất sau khi app đóng.
- [ ] Verified thủ công trên ít nhất macOS + 1 platform khác — **CHƯA đạt**, chỉ macOS được test. Theo đúng tiêu chí gốc của mục này, phase KHÔNG được coi là "hoàn thành" đầy đủ tới chuẩn merge ban đầu đề ra — chỉ Unix/macOS path có bằng chứng thực tế.

## Risk Assessment

- **Cross-platform PTY risk (CAO, rủi ro cao nhất toàn plan)**: forkpty/ConPTY khác biệt hành vi (buffering, signal delivery, console mode legacy Windows 10) — báo cáo 02 xác nhận đây là "high-risk area" cụ thể, không phải suy đoán.
- **Async boundary risk (CAO)**: `parking_lot::RwLock` shared state giữa PTY thread và GPUI task dispatch (theo báo cáo 02) — nếu bridge sai có thể deadlock hoặc race condition khó debug, đặc biệt trên GPUI's single-threaded-per-window model.
- **Resource leak risk (Trung bình)**: PTY process không cleanup đúng khi panel đóng đột ngột (crash, force-quit) — cần test explicit teardown path, không chỉ happy-path.
- **Giảm thiểu chung**: implement Unix trước (ít `#[cfg]` hơn, alacritty_terminal's Unix path trưởng thành hơn), Windows sau khi Unix path đã ổn định và có test coverage thủ công.

## Security Considerations

- PTY cho phép thực thi shell command tùy ý — đây LÀ tính năng cốt lõi (terminal), không phải lỗ hổng, nhưng cần đảm bảo: không tự động chạy lệnh nào không do user gõ (không auto-exec từ config/file ngoài).
- Hyperlink parsing trong terminal output (OSC 8 sequences) — nếu implement, phải sanitize trước khi render làm clickable link (tránh injection URL độc hại tự động mở khi user chỉ click xem output, không phải chủ động paste link).
- Không lưu terminal scrollback ra đĩa (base không có `crates/db` — nếu thêm sau, cần threat-model riêng cho việc lưu command history có thể chứa secret).

## Next steps

Đây là phase cuối cùng của plan này. Sau khi hoàn thành, review lại toàn bộ 3 phase với `code-reviewer` agent trước khi coi enrichment "hoàn tất" — đặc biệt kiểm tra lại platform-isolation convention không bị vi phạm qua cả 3 phase.
