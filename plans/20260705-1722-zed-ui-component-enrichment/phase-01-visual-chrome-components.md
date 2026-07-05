# Phase A — Visual Chrome Components

## Context links

- Plan overview: [plan.md](./plan.md)
- Research: [researcher-01-workspace-chrome.md](./research/researcher-01-workspace-chrome.md), [researcher-02-editor-terminal-stack.md](./research/researcher-02-editor-terminal-stack.md)
- Next phase: [phase-02](./phase-02-real-syntax-highlighting-and-terminal-chrome.md)

## Overview

- Date: 2026-07-05
- Description: Thêm picker/tab-switcher/breadcrumbs/title-bar chrome + nâng cấp gutter code_editor + hoàn thiện pane-group/dock chrome tối giản. Toàn bộ là component thuần GPUI (props-in, callback-out), không backing state phức tạp, không dependency mới.
- Priority: P1 (nền tảng cho Phase B/C)
- Implementation status: Completed (2026-07-05)
- Review status: Not Reviewed

**Ghi chú triển khai thực tế** (khác nhẹ so với đề xuất ban đầu):
- `breadcrumb.rs` đã đủ (path segments + click + separator) → không tạo `breadcrumb_bar.rs`, không đổi gì.
- ADR mục 1 chốt: giữ nguyên `command_palette`, viết `TabSwitcher` độc lập (không query input, chỉ ↑/↓/Tab/Shift-Tab/Enter/Esc) — không generic hóa `Picker<T>`.
- `PaneGroup` cài đặt dạng **flat entity** (mảng `fractions` + N-1 handle điều chỉnh cặp liền kề), KHÔNG lồng nhiều `ResizablePanelGroup` — lồng entity sẽ bị tạo lại mỗi frame trong closure, mất state kéo-thả. Tái dùng `ResizablePanel`/`ResizableHandle` sẵn có.
- `code_editor.rs`: `TextInput` không có cursor tracking thật (chỉ append-cuối) → "current-line highlight" chỉ có thể là dòng cuối cùng khi focus, đã ghi rõ giới hạn này trong doc comment thay vì giả vờ có full caret support.
- Đã verify thủ công: chạy `examples/ui_gallery` với trang mặc định lần lượt là Layout và Overlays — cả 3 component mới (`TitleBar`, `PaneGroupPreview`, `TabSwitcher`) render không panic.
- `make fmt-check` + `make check-all` pass trên toàn workspace.

## Key Insights (từ nghiên cứu)

- `picker` (Zed): 3.8 kLOC, coupling MEDIUM theo báo cáo 01 — nhưng `others/zed/crates/picker/Cargo.toml` cho thấy nó depend trực tiếp `workspace`, `db`, `language`, `project`, `zed_actions`, `ui_input`. **Không compile được nếu vendor thẳng.** Chỉ lấy state-machine ý tưởng (`picker.rs:1-60`).
- `tab_switcher` (1.5 kLOC) và `breadcrumbs` (127 LOC): cùng vấn đề — `Cargo.toml` của cả hai đều kéo `workspace.workspace = true` (`tab_switcher` còn kéo `editor`, `fuzzy_nucleo`, `project`). Base không có các crate này và sẽ không port chúng (xem plan.md Ngoài phạm vi) → chrome này phải viết lại từ đầu bằng GPUI primitive của base, không "port" theo nghĩa copy file.
- Base đã có `command_palette/palette.rs` (279 dòng) tự viết — generic modal picker (query input + fuzzy `score()` từ `fuzzy.rs`, không dùng external crate vì "corpus chỉ vài chục command"). Đây LÀ điểm khởi đầu tốt hơn Zed's picker cho quy mô base.
- `title_bar` (2.9 kLOC) + `platform_title_bar` (1.2 kLOC): coupling MEDIUM-HIGH, `title_bar/Cargo.toml` kéo `call`, `client`, `channel`, `auto_update`, `recent_projects` — toàn bộ collab/update stack. Chỉ port phần chrome thuần túy (border, nút traffic-light macOS, title text), bỏ hẳn app-menu/collab/onboarding-banner.
- Base đã có sẵn `tab.rs` (201 dòng, 2 style Underline/Pills), `tab_bar.rs` (262 dòng), `resizable.rs` (259 dòng, drag-resize 2-panel horizontal), `sidebar.rs` (138 dòng, static nav rail). Zed's `workspace` (48 kLOC dock/pane/pane_group/item) bị SKIP theo khuyến nghị báo cáo 01 — Phase A chỉ tự thiết kế 1 `PaneGroup`/`Dock` tối giản (N-panel chia dọc/ngang dùng lại `ResizablePanelGroup` pattern, không có tab-drag-to-split, không có item persistence).

## Requirements

1. `Picker<T>` component generic: query input + fuzzy-filtered list + keyboard nav (↑/↓/Enter/Esc), tách khỏi `CommandPalette` để dùng chung cho tab-switcher/go-to-line sau này — HOẶC giữ nguyên `command_palette` và chỉ thêm `TabSwitcher` như bản sao độc lập (xem ADR mục 1, quyết định thuộc về user).
2. `TabSwitcher` — overlay Cmd+Tab-style, danh sách item (label + icon + subtitle) + phím tắt điều hướng.
3. `Breadcrumbs` — hàng path segment clickable, dùng `IconName` cho separator, callback `on_click(segment_index)`.
4. Nâng cấp `code_editor.rs`: gutter hiện chỉ liệt kê `1..=line_count` tĩnh dựa trên đếm `\n` — không có current-line highlight, không click-to-place-cursor trên gutter. Thêm: highlight dòng hiện tại (dựa theo vị trí con trỏ của `TextInput`), width gutter tự co giãn theo số chữ số (hiện `GUTTER_WIDTH` cố định `px(44.)` — sai với file >999 dòng).
5. `PaneGroup`/`Dock` chrome tối giản: container chia N panel (dùng lại `ResizablePanelGroup` cho 2-panel, mở rộng generic N-panel nếu cần), mỗi panel có `TabBar` ở trên. Không tab-drag, không cross-pane drag, không persistence.
6. `TitleBar` chrome cơ bản: dải trên cùng có title text + 3 nút macOS-style (đóng/thu nhỏ/phóng to) render thuần túy — không gọi API cửa sổ thật (đó là việc của app, không phải `crates/ui`).

## Architecture

Không cần crate mới bắt buộc cho Phase A — mọi thứ nằm trong `crates/ui/src/components/`, theo đúng convention hiện tại của thư mục này (mỗi component 1 file hoặc 1 thư mục con như `command_palette/`).

```
crates/ui/src/components/
├── picker/                     (MỚI, chỉ nếu chọn refactor theo ADR mục 1)
│   ├── mod.rs                  generic Picker<T: PickerDelegate>
│   └── delegate.rs             trait PickerDelegate { fn match_text, fn render_item, ... }
├── tab_switcher.rs              (MỚI) overlay dùng Picker hoặc List độc lập
├── breadcrumb_bar.rs             (MỚI) — lưu ý: base ĐÃ CÓ breadcrumb.rs (kiểm tra trùng tên trước khi tạo)
├── pane_group.rs                 (MỚI) N-panel chrome, dùng ResizablePanelGroup
├── title_bar.rs                  (MỚI) chrome-only title bar
├── code_editor.rs                (SỬA) thêm current-line highlight + dynamic gutter width
├── tab_bar.rs, tab.rs, resizable.rs, sidebar.rs  (GIỮ NGUYÊN, tái sử dụng)
```

Lưu ý: `crates/ui/src/components/breadcrumb.rs` đã tồn tại (kiểm tra `ls crates/ui/src/components` → có `breadcrumb.rs`). Đặt tên `breadcrumb_bar.rs` cho component mới HOẶC audit `breadcrumb.rs` hiện tại xem đã đáp ứng yêu cầu path-navigation chưa trước khi tạo file mới trùng lặp (DRY).

Dependency graph không đổi: mọi component mới chỉ phụ thuộc `gpui`, `theme`, `icons` — giống các component hiện có, không thêm workspace dependency nào.

## ADR Rationale

**1. Giữ `command_palette` hiện tại hay refactor theo Picker/delegate pattern của Zed?**
- Context: Zed's `picker` (3.8kLOC) là abstraction generic dùng chung cho file-finder/command-palette/go-to-line/tab-switcher qua `PickerDelegate` trait.
- Decision: Phase A KHÔNG bắt buộc refactor. Chỉ refactor nếu roadmap thực sự cần ≥2 use-case picker khác nhau (tab-switcher là use-case thứ 2 trong chính phase này) — nếu vậy, trích xuất `PickerDelegate` trait tối thiểu (`fn matches(&self, query) -> Vec<usize>`, `fn render_item(&self, ix, selected) -> AnyElement`, `fn confirm(&mut self, ix)`) từ `command_palette/palette.rs` hiện tại, giữ `fuzzy.rs`'s `score()` (không đổi).
- Why this over alternatives: Vendor thẳng Zed's `picker` bất khả thi (coupling `workspace`/`db`/`project` như đã nêu ở Key Insights). Viết lại theo tinh thần "generic trait, nhẹ" tốn ít effort hơn refactor toàn bộ ngay từ đầu khi chỉ có 1 use-case (command_palette). Quyết định cuối: nếu user chỉ cần `TabSwitcher` một lần, sao chép pattern của `palette.rs` thành file riêng rẻ hơn là generic hóa sớm (YAGNI).

**2. Pane-group/Dock: tự thiết kế thay vì port Zed's `workspace` (48kLOC)?**
- Context: Báo cáo 01 khuyến nghị SKIP hẳn `workspace` — "porting defeats purpose", coupling VERY HIGH (`client::Client`, `project::Project`, LSP capability).
- Decision: Chỉ lấy Ý TƯỞNG (Dock = panel container có thể chia N vùng; Pane = 1 vùng chứa TabBar + content) — tự code bằng `ResizablePanelGroup` đã có (259 dòng, drag-resize 2-panel với `on_drag`/`on_drag_move`, đã test pattern này ổn định qua `redistributable_columns.rs`/`data_table.rs`).
- Why: `ResizablePanelGroup` hiện tại hard-code 2 panel (`left`/`right` closures). Mở rộng thành N-panel là refactor tự nhiên, không cần port gì từ Zed. Item persistence (Zed's `workspace::Item::serialize`) nằm ngoài phạm vi — base không có `crates/db` (theo `docs/system-architecture.md` § Add Persistence, chỉ thêm khi cần).

**3. Title bar: chrome-only, bỏ mọi state Zed-specific**
- Context: `title_bar/Cargo.toml` kéo `call`, `client`, `channel`, `auto_update`, `recent_projects` — toàn bộ product surface của Zed (collaboration presence, update banner, recent-project switcher).
- Decision: Port duy nhất phần render tĩnh (chiều cao, viền, vị trí 3 nút macOS, text ở giữa). Không có nút thật gọi window API — đó là trách nhiệm `crates/app` (dùng `gpui_platform` facade để lấy window handle thật), `crates/ui`'s `TitleBar` chỉ nhận callback `on_close`/`on_minimize`/`on_maximize` như prop.
- Why: Giữ platform-isolation convention của base (`docs/code-standards.md`: "no `#[cfg(target_os)]` ngoài platform crates") — `crates/ui` không được biết về Cocoa/Win32 API thật.

## Related code files

**Base (đọc/sửa):**
- `crates/ui/src/components/code_editor.rs` — sửa gutter
- `crates/ui/src/components/command_palette/palette.rs`, `fuzzy.rs` — tham khảo pattern, có thể trích `PickerDelegate`
- `crates/ui/src/components/tab.rs`, `tab_bar.rs` — tái dùng cho pane-group's per-pane tab strip
- `crates/ui/src/components/resizable.rs` — mở rộng N-panel
- `crates/ui/src/components/sidebar.rs`, `breadcrumb.rs` — audit trước khi thêm file mới trùng chức năng
- `crates/ui/src/components/tree_view_item.rs` — tham khảo nếu pane cần nội dung dạng cây (không bắt buộc Phase A)
- `crates/ui/src/lib.rs`, `crates/ui/src/components.rs` — đăng ký module mới
- `examples/ui_gallery/src/main.rs` — thêm preview cho component mới (theo convention `Component::preview()` đã thấy ở `Tab`/`TabBar`/`ResizablePreview`)

**Zed vendor (chỉ đọc, KHÔNG import trực tiếp):**
- `others/zed/crates/picker/src/picker.rs` (state machine, dòng 1-60), `render.rs`
- `others/zed/crates/tab_switcher/src/tab_switcher.rs`
- `others/zed/crates/breadcrumbs/src/breadcrumbs.rs`
- `others/zed/crates/title_bar/src/title_bar.rs`, `others/zed/crates/platform_title_bar/`

## Implementation Steps

1. Audit `crates/ui/src/components/breadcrumb.rs` hiện có — nếu đã đủ (path segments + click), bỏ qua bước breadcrumb, chỉ note trong changelog.
2. Quyết định ADR mục 1 (giữ nguyên hay generic hóa `command_palette`) trước khi code `TabSwitcher` — tránh viết 2 lần.
3. Implement `TabSwitcher` (hoặc `Picker<T>` + `TabSwitcher` dùng nó nếu chọn generic hóa).
4. Sửa `code_editor.rs`: gutter width = `f(line_count.to_string().len())`, current-line highlight bg dựa trên cursor row của `TextInput` (kiểm tra API `TextInput` đã expose cursor position public chưa — nếu chưa, thêm getter nhỏ, không đổi hành vi input).
5. Mở rộng `ResizablePanelGroup` → N-panel hoặc tạo `PaneGroup` mới bọc nhiều `ResizablePanelGroup` lồng nhau (đơn giản hơn, tái dùng 100% code hiện có — ưu tiên cách này theo KISS).
6. Implement `TitleBar` chrome-only.
7. Đăng ký toàn bộ component mới vào `crates/ui/src/components.rs` + `Component` catalog (theo pattern `RegisterComponent` đã thấy ở `Tab`/`TabBar`).
8. Thêm preview trong `examples/ui_gallery`.
9. `make fmt-check` + `make check-all`.

## Todo list

- [x] Audit `breadcrumb.rs` hiện có, quyết định giữ/mở rộng/không đổi — giữ nguyên, đã đủ
- [x] Chốt ADR mục 1 (picker generic hay không) — giữ `command_palette`, `TabSwitcher` độc lập
- [x] `TabSwitcher` component + preview
- [x] `code_editor.rs` dynamic gutter width + current-line highlight
- [x] `PaneGroup`/N-panel chrome + preview
- [x] `TitleBar` chrome-only + preview
- [x] Đăng ký module trong `components.rs`, `prelude.rs`
- [x] `make fmt-check && make check-all` pass
- [ ] Cập nhật `docs/codebase-summary.md` (component list) nếu tuân theo `documentation-management.md` — chưa làm, để lại cho lần cập nhật docs kế tiếp

## Success Criteria

- Tất cả component mới thuần `Render`/`RenderOnce`, không method nào gọi platform API trực tiếp (`cx.platform()` chỉ ở `crates/app` nếu cần).
- Không thêm workspace dependency mới trong `Cargo.toml` gốc.
- Mọi file mới < 200 dòng (theo `docs/code-standards.md`); nếu vượt, tách submodule.
- `cargo check --workspace --all-targets` và `cargo fmt --all -- --check` pass.
- Preview render được trong `examples/ui_gallery` không panic.

## Risk Assessment

- **Thấp tổng thể.** Rủi ro chính: trùng lặp chức năng với `breadcrumb.rs`/`command_palette` sẵn có nếu không audit trước (vi phạm DRY) — giảm thiểu bằng bước Implementation Step 1-2.
- Rủi ro nhỏ: `TextInput` có thể chưa expose cursor row/column public — cần kiểm tra API trước khi cam kết "current-line highlight" (nếu thiếu, thêm getter tối thiểu, không đổi behavior).

## Security Considerations

Không có bề mặt tấn công mới — component thuần render, không I/O, không network, không parse dữ liệu ngoài.

## Next steps

Sau khi Phase A hoàn thành và review, đọc [phase-02](./phase-02-real-syntax-highlighting-and-terminal-chrome.md) để quyết định có tiếp tục kéo `rope` + tree-sitter hay dừng ở chrome-only.
