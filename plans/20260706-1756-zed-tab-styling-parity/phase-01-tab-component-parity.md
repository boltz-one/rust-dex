# Phase 01 — Port TabPosition + rewrite Tab Underline về Zed-exact

**File:** `base/crates/ui/src/components/tab.rs`
**Tham chiếu Zed:** `others/zed/crates/ui/src/components/tab.rs:8-176`

## Bối cảnh
boltz `Tab` hiện chỉ có `TabBarStyle` (Underline/Pills), KHÔNG có `TabPosition`/`TabCloseSide`. Nhánh Underline đang là VSCode-style (top-accent). Cần port cơ chế position của Zed + rewrite nhánh Underline sang bg-based active + border-đáy inactive.

## Yêu cầu
1. Thêm 2 enum (port từ Zed `tab.rs:11-30`) + 2 hằng slot size:
```rust
use std::cmp::Ordering;
const START_TAB_SLOT_SIZE: Pixels = px(12.);
const END_TAB_SLOT_SIZE: Pixels = px(14.);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TabPosition { First, Middle(Ordering), Last }

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TabCloseSide { Start, End }
```
2. Thêm field `position: TabPosition` (default `First`) + `close_side: TabCloseSide` (default `End`) vào struct `Tab` + builder `.position()` / `.close_side()`. Export enum ở `components/tab.rs` mod + re-export nơi `Tab` được export (grep `pub use ...Tab`).
3. Rewrite nhánh `TabBarStyle::Underline` trong `render` bám sát Zed `tab.rs:109-179`:
```rust
TabBarStyle::Underline => {
    let (text_color, tab_bg) = if self.selected {
        (cx.theme().colors().text, cx.theme().colors().tab_active_background)
    } else {
        (cx.theme().colors().text_muted, cx.theme().colors().tab_inactive_background)
    };

    // start/end slot bọc kích thước cố định, hoán theo close_side
    let start = h_flex().size(START_TAB_SLOT_SIZE).justify_center().children(self.start_slot);
    let end = h_flex().size(END_TAB_SLOT_SIZE).justify_center().children(self.end_slot);
    let (start_slot, end_slot) = match self.close_side {
        TabCloseSide::End => (start, end),
        TabCloseSide::Start => (end, start),
    };

    self.div
        .h(Tab::container_height(cx))
        .bg(tab_bg)
        .border_color(cx.theme().colors().border)
        .map(|this| match self.position {
            TabPosition::First => if self.selected {
                this.pl_px().border_r_1().pb_px()
            } else { this.pl_px().pr_px().border_b_1() },
            TabPosition::Last => if self.selected {
                this.border_l_1().border_r_1().pb_px()
            } else { this.pl_px().border_b_1().border_r_1() },
            TabPosition::Middle(Ordering::Equal) => this.border_l_1().border_r_1().pb_px(),
            TabPosition::Middle(Ordering::Less) => this.border_l_1().pr_px().border_b_1(),
            TabPosition::Middle(Ordering::Greater) => this.border_r_1().pl_px().border_b_1(),
        })
        .cursor_pointer()
        .child(
            h_flex()
                .group("")
                .relative()
                .h(Tab::content_height(cx))
                .px(DynamicSpacing::Base04.px(cx))
                .gap(DynamicSpacing::Base04.rems(cx))
                .text_color(text_color)
                .child(start_slot)
                .children(self.children)   // title Label (Small) do Pane truyền
                .child(end_slot),
        )
}
```
   - **Bỏ**: `min_w(px(140.))`, `border_t_2()`, `palette::primary`, `semantic::elevated_surface`, `.hover(text_color)`, `px_3`, và bọc `flex_1().min_w_0()` (Zed không dùng — tab co theo nội dung; end_slot 14px đã đủ tách ✕).
   - Giữ nguyên nhánh `TabBarStyle::Pills` KHÔNG đổi.
4. Cập nhật `preview()`: các `Tab::new(...)` Underline nên set `.position(...)` minh hoạ First/Middle/Last (optional, để gallery đúng).

## Chú ý / rủi ro
- `.px(DynamicSpacing::Base04.px(cx))` trả `Pixels`; `.gap(DynamicSpacing::Base04.rems(cx))` trả `Rems` — bám đúng chữ ký như Zed (px cho padding, rems cho gap). Verify signature `DynamicSpacing` trong `base/crates/ui/src/styles/spacing.rs`.
- `transparent_black` import có thể thành dead nếu Pills không dùng → kiểm tra, xoá import nếu warning.
- Nhánh Pane render truyền `.child(Label::new(title).size(LabelSize::Small))` (giữ Small) — KHÔNG đụng ở đây; Tab chỉ nhận children.

## Success criteria
- Compile sạch (`cargo build -p boltz-ui`), không warning mới.
- `Tab` có API `.position()`/`.close_side()`; enum export ra ngoài crate.
- Underline render: active bg=`tab_active_background` + không border đáy; inactive bg=`tab_inactive_background` + border đáy; không accent xanh, không min-width.
