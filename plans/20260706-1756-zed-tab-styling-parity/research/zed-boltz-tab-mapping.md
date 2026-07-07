# Bảng mapping styling tab: Zed → boltz-ui

Nguồn: scout đọc trực tiếp `others/zed/crates/{ui,workspace,terminal_view,theme}` + `base/crates/{ui,theme}`.
Đơn vị @ UI density Default: `Base04=4px`, `Base06=6px`, `Base32=32px`, `1rem=16px`.

## Phát hiện cốt lõi
boltz `Tab` (v0.2.8) đã **đi lệch khỏi Zed** sang kiểu VSCode (`border_t_2` accent xanh `#3b82f6` + `elevated_surface` bg + `min_w(140)` + `px_3`). Zed đích thực KHÔNG có accent; phân biệt active bằng **bg sáng hơn + bỏ border đáy** trên nền TabBar có 1 đường kẻ đáy full-width. Theme boltz **đã có sẵn đúng token Zed** → mapping khả thi, không cần thêm token màu.

## Quyết định phạm vi (user chốt 2026-07-06)
- **Full parity 1:1 Zed**: bỏ top-accent, bỏ `min_w(140)`, ✕ chỉ hiện khi hover (kể cả tab active), tab co theo nội dung.
- **KHÔNG** thêm icon terminal vào tab (Zed có, ta bỏ) → tab chỉ có title.
- **Giữ `LabelSize::Small`** (Zed dùng Default 14px, ta giữ Small chống clip).

## Bảng token màu (đã xác nhận tồn tại trong `base/crates/theme/src/styles/colors.rs`)
| Vai trò | Zed token | boltz truy cập | boltz đang dùng (SAI) |
|---|---|---|---|
| TabBar bg | `tab_bar_background` | `cx.theme().colors().tab_bar_background` | `semantic::surface` (=surface_background) |
| Tab active bg | `tab_active_background` | `cx.theme().colors().tab_active_background` | `semantic::elevated_surface` |
| Tab inactive bg | `tab_inactive_background` | `cx.theme().colors().tab_inactive_background` | `transparent_black()` |
| Text active | `text` | `semantic::text` / `.colors().text` | `semantic::text` ✅ |
| Text inactive | `text_muted` | `semantic::text_muted` | `semantic::text_muted` ✅ |
| Viền tab + kẻ đáy | `border` | `cx.theme().colors().border` | (không dùng — dùng accent thay) |
| Accent (BỎ) | — (Zed không có) | — | `palette::primary(500)` `#3b82f6` ❌ bỏ |

## Bảng kích thước / layout
| Thuộc tính | Zed (đích) | boltz hiện tại | Hành động |
|---|---|---|---|
| Container height | `Base32`=32px | 32px ✅ | giữ |
| Content height | `Base32-1`=31px | 31px ✅ | giữ |
| Padding ngang inner | `px(Base04)`=4px | `px_3`=12px | **đổi → Base04** |
| Gap content | `gap(Base04)`=4px | `gap_2`=8px | **đổi → Base04** |
| min-width | KHÔNG | `min_w(140)` | **bỏ** |
| start slot | 12px vuông | (không có khái niệm) | **thêm** (dù rỗng) |
| end slot | 14px vuông | end_slot tự do | **bọc 14px** |
| Border active | `pb_px` + `border_l_1/r_1` (theo position), màu `border` | `border_t_2` accent | **thay bằng logic position Zed** |
| Border inactive | `border_b_1` (+pl/pr_px theo position) | transparent | **thay bằng logic position Zed** |

## TabBar (tab_bar.rs) mapping
| Thuộc tính | Zed | boltz | Hành động |
|---|---|---|---|
| bg | `tab_bar_background` | `semantic::surface` | **đổi → tab_bar_background** |
| Đường kẻ đáy | absolute full-width `border_b_1` màu `border` (overlay sau tabs) | KHÔNG (borderless) | **thêm overlay đáy** |
| Gap tabs | 0 (flush) | 0 (Underline) ✅ | giữ |
| start/end slot | `gap Base04 px Base06` + `border_b_1` + `border_r_1`(start)/`border_l_1`(end) màu `border` | `gap Base04 px Base06`, KHÔNG border | **thêm border_b_1 + r_1/l_1** |

## Close button (pane/render.rs) mapping
| Thuộc tính | Zed | boltz | Hành động |
|---|---|---|---|
| Icon | `IconName::Close` | `IconName::Close` ✅ | giữ |
| Icon size | `IconSize::Small`=14px | `XSmall`=12px | **đổi → Small** (khớp Zed) |
| Color | `Color::Muted` | (default) | **set Muted** |
| Shape | `IconButtonShape::Square` | (default) | **set Square** |
| Button size | `ButtonSize::None` | (default) | **set None** |
| Visible | hover-only (mọi tab) | active-luôn / inactive-hover | **bỏ nhánh `if !selected`** → hover-only tất cả |
| Vị trí | end_slot (close_side End) | end_slot ✅ | giữ |

## TabPosition (cần port từ Zed `tab.rs:11-30`)
```rust
pub enum TabPosition { First, Middle(std::cmp::Ordering), Last }
pub enum TabCloseSide { Start, End }
```
Border map theo Zed `tab.rs:148-166`:
- First+active: `pl_px().border_r_1().pb_px()` | First+inactive: `pl_px().pr_px().border_b_1()`
- Last+active: `border_l_1().border_r_1().pb_px()` | Last+inactive: `pl_px().border_b_1().border_r_1()`
- Middle(Equal)=active: `border_l_1().border_r_1().pb_px()`
- Middle(Less): `border_l_1().pr_px().border_b_1()` | Middle(Greater): `border_r_1().pl_px().border_b_1()`
- pane/render.rs tính: `position = ix.cmp(&active_idx)` cho Middle; First nếu `ix==0`, Last nếu `ix==len-1`.

## Ràng buộc quy trình (memory: crates.io = SOURCE OF TRUTH)
- Sửa trong `base/crates/ui` → validate trong `base` (ui_gallery + screenshot) → **bump version + publish crates.io (BẤT KHẢ NGHỊCH, cần user go-ahead)** → `terminal/` bump dep → verify.
- KHÔNG path-depend base cục bộ để test terminal.
- Skill dự án `gpui-ui-design` áp dụng khi sửa file trong `base/`.
