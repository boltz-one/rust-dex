---
title: Zed Tab Styling Parity — map tab của Zed sang terminal/ tabs
date: 2026-07-06 17:56
lane: high-risk (publish crates.io bất khả nghịch)
complexity: hard (cross-module base→crates.io→terminal, outward-facing publish)
status: planned
---

# Zed Tab Styling Parity

## Mục tiêu
Map **chính xác** styling tab mà Zed đang triển khai (`others/zed/crates/ui/src/components/{tab,tab_bar}.rs` + `workspace/src/pane.rs`) sang tab của `terminal/` — thực chất render qua boltz-ui `base/crates/ui/src/components/{tab.rs, tab_bar.rs, pane/render.rs}`, publish crates.io, `terminal/` consume.

## Bối cảnh (đã research)
boltz `Tab` v0.2.8 đã đi lệch sang kiểu VSCode (top-accent xanh, `elevated_surface`, `min_w(140)`, `px_3`). Zed đích thực: active = **bg sáng hơn + bỏ border đáy** trên nền TabBar có kẻ đáy full-width, KHÔNG accent. Theme boltz **đã có sẵn đúng token Zed** (`tab_active_background`/`tab_inactive_background`/`tab_bar_background`/`border`). Chi tiết: `research/zed-boltz-tab-mapping.md`.

## Phạm vi (user chốt)
- Full parity 1:1 Zed: bỏ top-accent, bỏ `min_w(140)`, ✕ hover-only (mọi tab), tab co theo nội dung, padding/gap = Base04.
- KHÔNG thêm icon terminal (tab chỉ title).
- Giữ `LabelSize::Small`.

## Ràng buộc
- **crates.io = SOURCE OF TRUTH**: validate trong base → publish (bất khả nghịch, cần go-ahead) → terminal consume. KHÔNG path-depend cục bộ.
- Skill `gpui-ui-design` áp dụng cho file `base/`.
- Không đụng nhánh `TabBarStyle::Pills` (chỉ sửa Underline — nhánh Pane dùng).

## Phases
| # | Phase | File | Status | Ownership |
|---|---|---|---|---|
| 01 | Port `TabPosition`/`TabCloseSide` + rewrite `Tab` Underline về Zed-exact | `base/crates/ui/src/components/tab.rs` | ⬜ | tab.rs |
| 02 | Rewrite `TabBar` Underline về Zed-exact (bg + kẻ đáy + slot borders) | `base/crates/ui/src/components/tab_bar.rs` | ⬜ | tab_bar.rs |
| 03 | Update `Pane::render` — TabPosition, close-button Zed-spec, hover-only | `base/crates/ui/src/components/pane/render.rs` | ⬜ | pane/render.rs |
| 04 | Validate trong base (ui_gallery + `make check-all` + screenshot) | `base/examples/ui_gallery/**` | ⬜ | tests/gallery |
| 05 | Bump boltz-ui 0.2.8→0.2.9 + publish crates.io ⚠️ GATE go-ahead | `base/crates/ui/Cargo.toml`, publish-crates.sh | ⬜ | version/publish |
| 06 | `terminal/` consume 0.2.9 + rebuild + screenshot-verify | `terminal/Cargo.toml` | ⬜ | terminal |

Thứ tự: 01→02→03 (song song được, cùng crate nên chú ý import chung), 04 sau khi 01-03 xong, 05 gate, 06 cuối.
Phase 01+02+03 nên cùng 1 developer (cùng crate `ui`, dùng chung `TabPosition`) để tránh xung đột.

## Success criteria
- Tab boltz-ui render giống Zed pixel-level: active=bg sáng + bỏ border đáy (không accent xanh), inactive=border đáy + text muted, padding 4px, không min-width, ✕ hover-only.
- `make check-all` + `fmt-check` xanh trong base.
- boltz-ui 0.2.9 publish crates.io + verify sparse index.
- `terminal/` chạy, screenshot xác nhận tab khớp Zed.

## Design decisions (đã chốt — best practice)
1. **✕ hover-only trên mọi tab (kể cả active)**: giữ nguyên — đây là mặc định đích thực của Zed (`show_close_button = "hover"`, `default.json:1344`), không phải lựa chọn tùy tiện. Nhất quán với full-parity.
2. **API close-button đã verify**: `.icon_color(Color::Muted).shape(IconButtonShape::Square).size(ButtonSize::None).icon_size(IconSize::Small)` — tất cả tồn tại trong boltz IconButton. Không còn rủi ro thiếu API. (chi tiết Phase 03)
3. **Kẻ đáy TabBar**: copy nguyên pattern Zed `div().absolute().top_0().left_0().size_full().border_b_1()` (KHÔNG dùng `right_0`). **Không thêm `.overflow_x_hidden()`** dù Zed có — memory chứng minh nó làm tab non-hit-testable; ưu tiên fix local. (chi tiết Phase 02)
4. **Không min-width + `LabelSize::Small`**: giữ no-min_w cho parity 1:1. Title terminal thường là path/command (dài) nên tab tự co giãn đủ rộng; tab quá hẹp chỉ xảy ra với title cực ngắn. **Best practice: tin theo Zed (no min_w) + kiểm chứng bằng screenshot ở Phase 04**; chỉ thêm lại `min_w` (vd `px(60.)`) nếu screenshot lộ tab clip/quá hẹp thực sự — KHÔNG thêm phòng thủ trước (YAGNI).

## Unresolved questions
Không còn. Mọi quyết định thiết kế đã chốt ở trên; các rủi ro kỹ thuật đã verify trong code thực.
