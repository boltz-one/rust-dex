# Phase 03 — Update Pane::render (TabPosition + close-button Zed-spec)

**File:** `base/crates/ui/src/components/pane/render.rs`
**Tham chiếu Zed:** `others/zed/crates/workspace/src/pane.rs:2777-2905, 3005-3016`

## Bối cảnh
`Pane::render` tạo tab qua `Tab::new(...).toggle_state(selected).end_slot(close).child(Label)`. Cần: truyền `TabPosition`, sửa close-button về spec Zed (Small/Muted/Square/None), và **bỏ nhánh `if !selected`** để ✕ hover-only cho MỌI tab.

## Yêu cầu
1. **TabPosition** cho mỗi tab (trong vòng lặp `for ix in 0..self.tabs.len()`):
```rust
use std::cmp::Ordering;
use crate::{TabPosition, TabCloseSide, IconButtonShape, ButtonSize}; // thêm import cần
let len = self.tabs.len();
let position = if ix == 0 {
    TabPosition::First
} else if ix == len - 1 {
    TabPosition::Last
} else {
    TabPosition::Middle(ix.cmp(&active_idx))
};
```
   Rồi `Tab::new(...).position(position).close_side(TabCloseSide::End)...`.
   (Trường hợp 1 tab duy nhất: `ix==0` → First; Zed cũng vậy. Với len==1 thì is_first && is_last — theo Zed ưu tiên First.)
2. **Close button** — sửa spec khớp Zed `pane.rs:3006-3016`. **API đã verify tồn tại** trong boltz (`icon_button.rs:50` `.shape()`, `:60` `.icon_color()`, `:160` `.size()` qua trait `ButtonCommon`; `ButtonSize::None` = `button_like.rs:488`):
```rust
let mut close_ib = IconButton::new(("pane-tab-close", tab_id.0), IconName::Close)
    .icon_size(IconSize::Small)          // XSmall → Small (14px, khớp Zed)
    .icon_color(Color::Muted)            // ✅ có
    .shape(IconButtonShape::Square)      // ✅ có
    .size(ButtonSize::None)              // ✅ có (ButtonCommon)
    .on_click(cx.listener(move |this, _, _, cx| { this.close_tab(ix, cx); }));
```
   - Import: `IconButtonShape` re-export ở crate root → `use crate::IconButtonShape;` (verified: modal.rs/context_menu.rs dùng vậy). `ButtonSize` có trong `prelude`. `.size()` cần trait `ButtonCommon` trong scope — nếu chưa có, thêm `use crate::ButtonCommon;` (đã export ở button_like).
3. **Hover-only cho mọi tab**: XOÁ nhánh `if !selected { close_ib = close_ib.visible_on_hover(...) }`, thay bằng luôn set:
```rust
close_ib = close_ib.visible_on_hover(hover_group.clone());
```
   Cập nhật comment dòng 40-41 (bỏ "active always shows").
4. **Giữ nguyên**: `.child(Label::new(title).size(LabelSize::Small))` (user chốt Small), KHÔNG thêm icon terminal (user chốt không icon), drag/reorder handlers, nút `+` và close-pane ở `end_child`.
5. **min_w** đã bỏ ở tab.rs (Phase 01) → không cần đụng ở đây.

## Chú ý / rủi ro
- `active_idx` đã có sẵn (dòng 26). `ix.cmp(&active_idx)` khi `selected` = Ordering::Equal → Middle(Equal) render như active. Nhưng nếu tab active là First/Last thì nhánh First/Last+selected xử lý đúng (pb_px). OK.
- Sau khi ✕ hover-only: tab active KHÔNG còn ✕ thường trực → khác trải nghiệm 0.2.5. User đã chốt full-parity nên chấp nhận.
- `IconButtonShape`/`ButtonSize`/`ButtonCommon` ĐÃ export ở crate root (verified) — chỉ cần `use`, không cần thêm `pub use`.

## Success criteria
- Mỗi tab nhận đúng `TabPosition`; border render đúng theo vị trí + active/inactive.
- ✕ = 14px, Muted, Square, hover-only cho cả active lẫn inactive.
- Click/drag/reorder/nút +/close-pane vẫn hoạt động.
- `cargo build -p boltz-ui` sạch.
