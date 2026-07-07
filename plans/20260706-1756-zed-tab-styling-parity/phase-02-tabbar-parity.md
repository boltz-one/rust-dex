# Phase 02 — Rewrite TabBar Underline về Zed-exact

**File:** `base/crates/ui/src/components/tab_bar.rs`
**Tham chiếu Zed:** `others/zed/crates/ui/src/components/tab_bar.rs:92-152`

## Bối cảnh
boltz TabBar Underline hiện: bg=`semantic::surface`, KHÔNG có đường kẻ đáy (borderless), start/end slot không border. Zed: bg=`tab_bar_background`, có overlay kẻ đáy full-width `border_b_1` màu `border`, start/end slot có `border_b_1` + `border_r_1`(start)/`border_l_1`(end).

## Yêu cầu
1. **bg container**: đổi `semantic::surface(cx)` → `cx.theme().colors().tab_bar_background` (dòng 156).
2. **Đường kẻ đáy full-width** (nhánh Underline `middle`): thêm overlay absolute làm **child ĐẦU TIÊN** (paint trước → nằm dưới tabs), copy **nguyên văn Zed** `tab_bar.rs:114-128`:
```rust
TabBarStyle::Underline => div()
    .relative()
    .flex_1()
    .h_full()
    // Child đầu: overlay kẻ đáy (paint trước, nằm dưới tabs). Div KHÔNG có
    // .id()/on_click → không interactive → không chặn hit-test của tabs.
    .child(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .border_b_1()
            .border_color(cx.theme().colors().border),
    )
    .child(tabs_row)   // child sau: paint trên overlay, vẫn click được
    .into_any_element(),
```
   - **best-practice (đã verify)**: dùng đúng pattern Zed `absolute().top_0().left_0().size_full().border_b_1()` (KHÔNG dùng `right_0`/`.h(px(1))` như bản nháp cũ — Zed dùng `size_full()` + chỉ `border_b_1` để vẽ riêng mép đáy).
   - Lý do overlay thay vì `border_b_1` trên container: tab active dùng `pb_px` "khoét" qua kẻ đáy → kẻ đáy phải nằm SAU tabs, không phải border container.
   - ⚠️ **KHÁC Zed có chủ đích**: Zed bọc middle trong `.overflow_x_hidden()`, NHƯNG memory boltz đã chứng minh overflow clip ở wrapper này làm `Tab` children **non-hit-testable** (click no-op). → **GIỮ quyết định local: KHÔNG thêm `.overflow_x_hidden()`** (ưu tiên fix đã kiểm chứng của dự án hơn upstream). Overlay absolute không cần overflow clip để vẽ kẻ đáy.
3. **start slot** (dòng 157-165): thêm `.border_b_1().border_r_1().border_color(cx.theme().colors().border)` vào h_flex, giữ `gap Base04` + `px Base06`.
4. **end slot** (dòng 167-175): thêm `.border_b_1().border_l_1().border_color(cx.theme().colors().border)`, giữ `gap Base04` + `px Base06`.
5. Giữ height `Tab::container_height` (32px), `flex flex_none w_full`, gap tabs=0 (Underline flush). KHÔNG đụng nhánh Pills.

## Chú ý / rủi ro
- Cập nhật doc-comment dòng 128-133 ("No bottom border line") vì giờ CÓ kẻ đáy.
- Sau đổi bg sang `tab_bar_background`: ở nhiều theme `tab_bar_background == tab_inactive_background` (step_2) nên tab inactive "chìm" vào bar, active (step_1) nổi lên — đúng ý đồ Zed.
- Sau khi thêm kẻ đáy: chạy lại `pane_group_probe_e2e.rs` (Phase 04) để chắc chắn overlay không hồi quy hit-test (đây là bug từng gặp).

## Success criteria
- TabBar bg = `tab_bar_background`; có kẻ đáy 1px full-width màu `border`.
- start/end slot có viền đáy + viền trong; tab active nối liền content qua khoảng `pb_px`.
- Click tab vẫn hoạt động (không bị overlay chặn).
- `cargo build -p boltz-ui` sạch.
