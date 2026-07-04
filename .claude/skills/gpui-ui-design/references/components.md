# Component Catalog — base set

Verbatim constructors, the builders you'll actually use, and one idiomatic example — for the
**original Zed-derived component set** (Button/Toggle/Label/Icon/Tooltip/List/Modal/Popover/
Disclosure/Banner/Progress/Avatar). All are re-exported at the crate root (`ui::Button`,
`ui::Label`, …) and most via `ui::prelude::*`.

For the ~90-component **shadcn-parity kit** added on top (Select, Combobox, Dialog, Drawer,
Command, Calendar, DatePicker, Table, Tabs, Chart, Slider, Sonner, …) and the design-token
system, see **`references/design-system.md`** — that is the current, complete catalog + rules.
Source of truth for exact signatures either way: `crates/ui/src/components/<name>.rs` — each
file has a `preview()` fn with gold-standard examples.

Shared interaction traits (defined in `crates/ui/src/traits/`):
- `Clickable` → `.on_click(handler)`, `.cursor_style(cs)`
- `Disableable` → `.disabled(bool)`
- `Toggleable` → `.toggle_state(bool)`
- `SelectableButton` → `.selected_style(ButtonStyle)`
- `FixedWidth` → `.width(len)`, `.full_width()`
- `VisibleOnHover` → `.visible_on_hover(group_name)`
- `ButtonCommon` → `.id()`, `.style(ButtonStyle)`, `.size(ButtonSize)`, `.tooltip(view_fn)`, `.tab_index(i)`, `.layer(ElevationIndex)`, `.track_focus(&fh)`

Handler signature: `Fn(&ClickEvent, &mut Window, &mut App) + 'static` — wrap entity state with `cx.listener(|this, event, window, cx| { ... })`.

## Table of contents
1. Button · IconButton · CopyButton · SplitButton · ButtonLike
2. Toggle family — Checkbox · Switch · SwitchField · ToggleButtonGroup
3. Label · Headline · LoadingLabel
4. Icon · IconName · IconSize · DecoratedIcon · Indicator
5. Tooltip · KeyBinding · KeybindingHint
6. List · ListItem · ListHeader
7. Modal · ModalHeader · ModalFooter · Section
8. Popover · PopoverMenu · ContextMenu · DropdownMenu · RightClickMenu
9. Disclosure · Divider · Chip
10. Banner · Callout · AlertModal · AnnouncementToast
11. ProgressBar · CircularProgress
12. Avatar · Facepile · CountBadge · DiffStat · Vector(Image)
13. Colors · Severity · ButtonStyle · ButtonSize · TintColor

---

## 1. Buttons

### `Button` — label + optional icons (the primary button)
```rust
Button::new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self
```
Builders: `.color(Color)`, `.label_size(LabelSize)`, `.start_icon(Icon)`, `.end_icon(Icon)`, `.key_binding(KeyBinding)`, `.key_binding_position(KeybindingPosition)`, `.selected_label(s)`, `.truncate(bool)`, `.loading(bool)`, `.full_width()`, `.disabled(bool)`, `.tooltip(view_fn)`, `.aria_label(s)`.
Traits: `Clickable`, `Disableable`, `Toggleable`, `SelectableButton`, `ButtonCommon`, `FixedWidth`.
```rust
Button::new("save", "Save")
    .style(ButtonStyle::Filled)
    .start_icon(Icon::new(IconName::Save))
    .on_click(cx.listener(|this, _, _, cx| { this.save(cx); }));

// Toggle button that turns accent when selected
Button::new("toggle", "Toggle Me")
    .start_icon(Icon::new(IconName::Check))
    .toggle_state(true)
    .selected_style(ButtonStyle::Tinted(TintColor::Accent))
    .on_click(|_, _, _| {});

// Full-width button (forms / modal footers)
Button::new("confirm", "Confirm").full_width();
```

### `IconButton` — icon only (the most-used button in the codebase)
```rust
IconButton::new(id: impl Into<ElementId>, icon: IconName) -> Self
```
Builders: `.shape(IconButtonShape::{Square, Wide})` (default `Wide`), `.icon_size(IconSize)`, `.icon_color(Color)`, `.selected_icon(IconName)`, `.indicator(Indicator)`, `.on_right_click(h)`, `.aria_label(s)`, `.aria_expanded(bool)`. Plus all `ButtonCommon`/`Clickable`/`Toggleable`/`SelectableButton`/`Disableable`/`FixedWidth`/`VisibleOnHover`.
```rust
IconButton::new("settings", IconName::Settings)
    .style(ButtonStyle::Subtle)
    .layer(ElevationIndex::Background)
    .tooltip(Tooltip::text("Settings"))
    .on_click(cx.listener(|this, _, _, cx| { /* ... */ }));

IconButton::new("square-btn", IconName::Check)
    .shape(IconButtonShape::Square)
    .style(ButtonStyle::Filled);
```
**Always pass `.aria_label(...)` on an IconButton** — it has no visible text, so a11y needs a name.

### `CopyButton` — copy-to-clipboard with 2s "copied" state
```rust
CopyButton::new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self
```
Builders: `.icon_size(IconSize)`, `.disabled(bool)`, `.tooltip_label(s)` (default `"Copy"`), `.visible_on_hover(group)`, `.custom_on_click(h)` (overrides clipboard write).
```rust
CopyButton::new("copy-id", "abc-123").tooltip_label("Copy ID");
```

### `SplitButton` — primary action + secondary action
```rust
SplitButton::new(left: impl Into<SplitButtonKind>, right: AnyElement) -> Self
// left: an IconButton or ButtonLike
```
Builders: `.style(SplitButtonStyle::{Filled, Outlined, Transparent})` (default `Filled`).

### `ButtonLike` — build a fully custom button when Button/IconButton don't fit
`Button::new(...)` returns a `ButtonLike` from its `RenderOnce`. For custom shapes/compositions, construct `ButtonLike::new(id)` directly and add children/traits. See `crates/ui/src/components/button/button_like.rs`.

### `ButtonStyle` & `ButtonSize` & `TintColor`
```rust
pub enum ButtonStyle {
    Subtle,           // #[default] — transparent bg, hover/active states. The common button.
    Filled,           // solid bg — emphasis / primary CTA
    Tinted(TintColor),// semantic coloring (selected, error, success…)
    Outlined,         // secondary, more emphasis than transparent
    OutlinedGhost,    // de-emphasized outlined
    OutlinedCustom(Hsla),
    Transparent,      // only foreground changes on hover/active
}
pub enum ButtonSize { Default, Large, Compact, None, Custom(Rems) }
pub enum TintColor { Accent, Error, Warning, Success, Info }
```
The default button is `ButtonStyle::Subtle`. Use `Filled` for primary CTAs, `Transparent` for the lowest emphasis, `Tinted(TintColor::Accent)` for a selected/toggle state.

---

## 2. Toggle family

Free helpers: `checkbox(id, state)` and `switch(id, state)`.

### `Checkbox`
```rust
Checkbox::new(id: impl Into<ElementId>, checked: ToggleState) -> Self
```
Builders: `.disabled(bool)`, `.placeholder(bool)`, `.on_click(h: Fn(&ToggleState, &mut Window, &mut App))`, `.fill()`, `.style(ToggleStyle)`, `.label(s)`, `.label_size(LabelSize)`, `.label_color(Color)`, `.tooltip(...)`.
```rust
Checkbox::new("agree", ToggleState::Unselected)
    .label("I agree to the terms")
    .on_click(cx.listener(|this, state: &ToggleState, _, cx| { this.agreed = state.selected(); cx.notify(); }));
```

### `Switch`
```rust
Switch::new(id: impl Into<ElementId>, state: ToggleState) -> Self
```
Builders: `.color(SwitchColor)`, `.disabled(bool)`, `.on_click(h)`, `.label(s)`, `.label_position(...)`, `.label_size(LabelSize)`, `.full_width(bool)`, `.key_binding(...)`, `.aria_label(s)`.

### `SwitchField` — labeled toggle row (settings UI)
```rust
SwitchField::new(
    id, label: Option<impl Into<SharedString>>,
    description: Option<SharedString>,
    toggle_state: impl Into<ToggleState>,
    on_click: impl Fn(&ToggleState, &mut Window, &mut App),
) -> Self
```
Builders: `.description(s)`, `.disabled(bool)`, `.color(SwitchColor)`, `.tooltip(...)`, `.tab_index(i)`.

### `ToggleButtonGroup` — segmented control
```rust
ToggleButtonGroup::single_row(group_name, [T; COLS])   // T = ToggleButtonSimple | ToggleButtonWithIcon
ToggleButtonGroup::two_rows(group_name, row1: [T; COLS], row2: [T; COLS])
ToggleButtonSimple::new(label, on_click)
ToggleButtonWithIcon::new(label, icon: IconName, on_click)
```
Builders: `.style(ToggleButtonGroupStyle::{Transparent, Filled, Outlined})`, `.size(...)`, `.selected_index(usize)`, `.auto_width()`, `.label_size(LabelSize)`.
```rust
ToggleButtonGroup::single_row("align", [
    ToggleButtonWithIcon::new("Left", IconName::AlignLeft, |_, _, _| {}),
    ToggleButtonWithIcon::new("Center", IconName::AlignCenter, |_, _, _| {}),
    ToggleButtonWithIcon::new("Right", IconName::AlignRight, |_, _, _| {}),
]).selected_index(1)
```
`ToggleState` enum: `Unselected`, `Indeterminate`, `Selected` — has `.selected()`, `from_any_and_all(...)`, `From<bool>`.

---

## 3. Labels & text

### `Label`
```rust
Label::new(text: impl Into<SharedString>) -> Self
```
Builders: `.color(Color)`, `.size(LabelSize)`, `.weight(FontWeight)`, `.line_height_style(LineHeightStyle)`, `.strikethrough()`, `.italic()`, `.underline()`, `.alpha(f32)`, `.truncate()`, `.truncate_start()`, `.truncate_middle()`, `.line_clamp(n)`, `.single_line()`, `.inline_code(cx)`, `.render_code_spans()`, `.flex_1()`, `.flex_none()`, `.flex_grow()`.
```rust
Label::new("Build Failed").color(Color::Error).weight(FontWeight::BOLD);
Label::new("use `zed` to open").render_code_spans(); // backtick → monospace code span
```
`LabelSize`: `Default` (the default), `Small`, `Large`, `XSmall`, `Custom(Rems)`. `LineHeightStyle`: `TextLabel` (default) vs `UiLabel` (line-height 1, compact).

### `Headline`
For large titles (modal headers, empty states). `Headline::new(text).size(HeadlineSize::...)`.

### `LoadingLabel`
A label with a spinner; for async/loading states.

---

## 4. Icons

### `Icon`
```rust
Icon::new(icon: IconName) -> Self
Icon::from_path(path) -> Self
Icon::from_external_svg(svg) -> Self
```
Builders: `.color(Color)`, `.size(IconSize)`, `.with_keyed_rotate_animation(id, seconds)` (spinner), trait `Transformable::transform(...)`.
```rust
Icon::new(IconName::Star).size(IconSize::Small).color(Color::Accent);
```

### `IconName` (~265 variants)
Enum in `crates/icons/src/icons.rs`, snake_case serialized. **The exact spelling matters — many "obvious" names do NOT exist and will fail to compile.** Verified variants that DO exist (a useful subset): `Check`, `CheckDouble`, `Close`, `Trash`, `Settings`, `Bell`, `BellDot`, `BellOff`, `BellRing`, `Plus`, `Minus`, `ChevronUp`, `ChevronDown`, `ChevronLeft`, `ChevronRight`, `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`, `ArrowUpRight`, `ArrowDownRight`, `ArrowRightLeft`, `Copy`, `Download`, `Upload`, `CloudDownload`, `Warning`, `Info`, `CircleHelp`, `XCircle`, `XCircleFilled`, `Clock`, `Calendar`, `FileText`, `Folder`, `FolderOpen`, `FolderSearch`, `Image`, `Link`, `ExternalLink`, `Menu`, `Ellipsis`, `User`, `UserGroup`, `UserRoundPen`, `UserCheck`, `Star`, `Bookmark`, `Play`, `Pause`, `Stop`, `LoadCircle`, `DatabaseZap`, `Terminal`, `ToolTerminal`, `Debug`, `DebugBreakpoint`, `DebugContinue`, `GitBranch`, `Diff`, `Filter`, `Eye`, `EyeOff`, `Lock`, `Unlock`, `Key`, `Envelope`, `Message`, `Send`, `Reply`, `Pin`, `Unpin`, `Magnet`, `BoltFilled`, `BoltOutlined`, `Sparkles`, `Circle`, `Triangle`, `TriangleRight`, `ExpandDown`, `Eraser`, `Crosshair`, `CursorIBeam`, `WholeWord`, `CaseSensitive`, `RotateCcw`, `RotateCw`, `RefreshTitle`, `MagnifyingGlass`, `ToolSearch`, `ToolThink`, `ToolWeb`, `ToolTerminal`, `ZedAgent`, `ZedAssistant`, `ZedPredict`. AI variants: `AiAnthropic`, `AiClaude`, `AiGemini`, `AiOpenAi`, `AiOllama`, `AiZed`, `AiDeepSeek`, `AiMistral`.

**⚠️ Gotchas — these "obvious" names do NOT exist** (real evals failed on them):
- `Search` → use **`MagnifyingGlass`** (UI) or `ToolSearch`/`FolderSearch`.
- `Save` → there is **no save icon**; use `Download`, `CloudDownload`, or `Check`.
- `Sync` / `Refresh` → use **`RotateCcw`** (sync/undo) or `RefreshTitle`.
- `Moon` / `Sun` / `Dark` / `Light` → none exist; for theme toggle use `BoltOutlined` or a `Label`.
- `Activity` → use `SignalHigh`; `Volume` → use `AudioOn`/`AudioOff`.
- `Sort` / `SortAsc` / `SortDesc` → none exist; use `ArrowUp`/`ArrowDown` or `ChevronUp`/`ChevronDown`.

**Always verify before committing to an icon name:** `grep -E "^\s+(NameHere)," crates/icons/src/icons.rs`. The full list is the source of truth — don't trust memory.

### `IconSize`
```rust
pub enum IconSize { Indicator /*10px*/, XSmall /*12*/, Small /*14*/, Medium /*16, default*/, XLarge /*48*/, Custom(Rems) }
```

### `DecoratedIcon` / `IconDecoration`
Overlay an X / dot / triangle on an icon (e.g. "mute" badge on a mic).
```rust
let badge = IconDecoration::new(IconDecorationKind::X, knockout_color, cx)
    .color(palette::danger(500))
    .position(Point { x: px(-2.), y: px(-2.) });
DecoratedIcon::new(Icon::new(IconName::Mic), Some(badge))
```

### `Indicator` — small status dot/bar/icon
```rust
Indicator::dot() | Indicator::bar() | Indicator::icon(impl Into<AnyIcon>)
```
Builders: `.color(Color)`, `.border_color(Color)`.
```rust
Indicator::dot().color(Color::Success);
Indicator::dot().color(Color::Accent).border_color(Color::Default);
```

---

## 5. Tooltip · KeyBinding · KeybindingHint

### `Tooltip` (a view; pass as a closure to `.tooltip(...)`)
```rust
Tooltip::text(title) -> impl Fn(&mut Window, &mut App) -> AnyView   // most common
Tooltip::new(title: SharedString) -> Self
Tooltip::simple(title, cx) -> AnyView
Tooltip::for_action(title, &action, cx) -> AnyView
Tooltip::for_action_in(title, &action, &focus_handle, cx) -> AnyView
Tooltip::with_meta(title, action, meta, cx) -> AnyView
```
```rust
Button::new("delete", "Delete").tooltip(Tooltip::text("Delete the selected item"));
IconButton::new("search", IconName::Search)
    .tooltip(move |_, cx| Tooltip::for_action("Search", &DeploySearch, cx));
```

### `KeyBinding` — renders a keyboard shortcut chip
```rust
KeyBinding::for_action(&action, cx)                 // from the registered binding (preferred)
KeyBinding::for_action_in(&action, &focus_handle, cx)
KeyBinding::from_keystrokes(keystrokes: Rc<[KeybindingKeystroke]>, vim_mode: bool)
```
Builders: `.platform_style(PlatformStyle::{Mac, Linux, Windows})`, `.size(AbsoluteLength)`, `.disabled(bool)`, `.vim_mode(bool)`.
```rust
let kb = KeyBinding::for_action(&menu::Confirm, cx);
```

### `KeybindingHint` — KeyBinding + prefix/suffix label on a colored bg
```rust
KeybindingHint::new(kb, bg_color: Hsla)
KeybindingHint::with_prefix(prefix, kb, bg)
KeybindingHint::with_suffix(kb, suffix, bg)
```
Builders: `.prefix(s)`, `.suffix(s)`, `.size(Pixels)`.

---

## 6. List

### `List` (ParentElement)
```rust
List::new() -> Self   // also Default
```
Builders: `.empty_message(impl Into<EmptyMessage>)` (text or element), `.header(ListHeader)`, `.toggle(bool)`.
```rust
List::new()
    .child(ListItem::new("i1").child(Label::new("Item 1")))
    .child(ListItem::new("i2").child(Label::new("Item 2")))
List::new().empty_message("No items to display")
```

### `ListItem` (ParentElement)
```rust
ListItem::new(id: impl Into<ElementId>) -> Self
```
Key builders: `.selectable(has_hover: bool)`, `.on_click(h)`, `.on_hover(h: Fn(&bool, &mut Window, &mut App))`, `.tooltip(...)`, `.inset(bool)`, `.indent_level(usize)`, `.toggle(bool)`, `.start_slot<E>()`, `.end_slot<E>()`, `.end_slot_on_hover<E>()`, `.outlined()`, `.rounded()`, `.height(len)`, `.spacing(ListItemSpacing)`, `.group_name(s)`, `.aria_role(Role)`, `.aria_label(s)`, `.focused(bool)`.
```rust
ListItem::new("file")
    .selectable(true)
    .on_click(cx.listener(|this, _, _, cx| { /* open */ }))
    .start_slot(Icon::new(IconName::FileText))
    .child(Label::new("main.rs"))
    .end_slot(CopyButton::new("copy", "main.rs"))
```

### `ListHeader` / `ListSubHeader` / `ListBulletItem` / `ListSeparator`
```rust
ListHeader::new(label).toggle(bool).on_toggle(h).start_slot<E>().end_slot<E>()
ListSubHeader::new(label)
ListBulletItem::new(label)
ListSeparator  // unit struct, just drop in as a child
```

---

## 7. Modal

### `Modal` (ParentElement)
```rust
Modal::new(id: impl Into<SharedString>, scroll_handle: Option<ScrollHandle>) -> Self
```
Builders: `.header(ModalHeader)`, `.section(Section)`, `.footer(ModalFooter)`, `.show_dismiss(bool)`, `.show_back(bool)`. Children go into the scrollable body.

### `ModalHeader` (ParentElement)
```rust
ModalHeader::new() -> Self   // also Default
```
Builders: `.icon(Icon)`, `.headline(s)`, `.description(s)`, `.show_dismiss_button(bool)`, `.show_back_button(bool)`. Renders Close/Back IconButtons dispatching `menu::Cancel`.

### `ModalFooter`
```rust
ModalFooter::new().start_slot<E>().end_slot<E>()
```
Convention: `.end_slot(h_flex().gap_2().child(Button("cancel")).child(Button("confirm")))`.

### `Section` / `SectionHeader` (ParentElement)
```rust
Section::new() | Section::new_contained()
Section::contained(bool).header(SectionHeader).meta(s).padded(bool)
SectionHeader::new(label).end_slot<E>()
```
```rust
Modal::new("settings", None)
    .header(ModalHeader::new().headline("Settings"))
    .section(Section::new().header(SectionHeader::new("Appearance")))
    .footer(ModalFooter::new().end_slot(
        h_flex().gap_2()
            .child(Button::new("cancel", "Cancel").style(ButtonStyle::Subtle))
            .child(Button::new("save", "Save"))
    ))
```

---

## 8. Popovers & menus

### `Popover` — static positioned container (ParentElement, `RenderOnce`)
```rust
Popover::new() -> Self   // also Default
.aside(element) -> Self   // side panel
```
For "menu"-style surfaces anchored to a fixed spot, not the cursor. Children render in an elevated `v_flex`.

### `PopoverMenu<M: ManagedView>` — anchor a menu entity to a trigger
```rust
PopoverMenu::new(id: impl Into<ElementId>) -> Self
```
Builders: `.menu(f: Fn(&mut Window, &mut App) -> Option<Entity<M>>)`, `.trigger(t: impl PopoverTrigger)` (sets toggle + on_click), `.trigger_with_tooltip(t, tooltip_fn)`, `.anchor(Anchor)`, `.attach(Anchor)`, `.offset(Point<Pixels>)`, `.full_width(bool)`, `.with_handle(PopoverMenuHandle<M>)`, `.on_open(f)`.
`PopoverMenuHandle<M>`: `.show(window, cx)`, `.hide(cx)`, `.toggle(window, cx)`, `.is_deployed()`, `.refresh_menu(window, cx, builder)`.
```rust
PopoverMenu::new(("menu", "popover"))
    .menu(move |_, _| Some(my_menu_entity.clone()))
    .trigger(Button::new("open", "Open Menu").end_icon(Icon::new(IconName::ChevronDown)))
    .anchor(Anchor::BottomLeft)
```

### `ContextMenu` (Entity, `Focusable`, `EventEmitter<DismissEvent>`)
Build via factory — returns `Entity<Self>`:
```rust
ContextMenu::build(window, cx, |this, _, _| this.entry(...).separator()...)
ContextMenu::build_persistent(window, cx, builder)  // stays open after confirm; rebuildable
```
Builders (chain on `this`): `.header(s)`, `.label(s)`, `.separator()`, `.entry(label, action: Option<Box<dyn Action>>, handler)`, `.action(label, action)`, `.action_checked(label, action, checked)`, `.action_disabled_when(disabled, label, action)`, `.link(label, action)`, `.submenu(label, |menu, _, _| ...)`, `.submenu_with_icon(label, icon, ...)`, `.custom_entry(render_fn, handler, aside)`, `.context(focus)`, `.keep_open_on_confirm(bool)`, `.fixed_width(len)`, `.key_context(s)`, `.item(item)`, `.extend(items)`.
```rust
let menu = ContextMenu::build(window, cx, |this, _, _| {
    this.entry("Open", None, |_, _| {})
        .entry("Rename", None, |_, _| {})
        .separator()
        .submenu("Sort By", |m, _, _| {
            m.entry("Name", None, |_, _| {}).entry("Date", None, |_, _| {})
        })
        .separator()
        .action("Delete", Box::new(Delete))
});
```

### `DropdownMenu` — trigger button + ContextMenu via PopoverMenu (`Disableable`)
```rust
DropdownMenu::new(id, label: impl Into<SharedString>, menu: Entity<ContextMenu>) -> Self
DropdownMenu::new_with_element(id, label: AnyElement, menu) -> Self
```
Builders: `.style(DropdownStyle::{Solid, Outlined, Subtle, Ghost})` (default `Solid`), `.trigger_size(ButtonSize)`, `.trigger_tooltip(...)`, `.trigger_icon(IconName)` (default `ChevronUpDown`), `.full_width(bool)`, `.handle(PopoverMenuHandle<ContextMenu>)`, `.attach(Anchor)`, `.offset(Point<Pixels>)`, `.no_chevron()`, `.disabled(bool)`, `.aria_label(s)`.
```rust
DropdownMenu::new("sort", "Sort by", menu)
DropdownMenu::new("full", "Full Width", menu).full_width(true)
DropdownMenu::new("out", "Outlined", menu).style(DropdownStyle::Outlined).disabled(false)
```

### `right_click_menu` / `RightClickMenu<M>` — cursor-anchored
```rust
ui::right_click_menu::<M>(id: impl Into<ElementId>) -> RightClickMenu<M>
```
Builders: `.menu(f)`, `.maybe_menu(f -> Option<Entity>)`, `.trigger(F: FnOnce(bool /*active*/, window, cx) -> E)`, `.anchor(Anchor)`, `.attach(Anchor)`. Default position = cursor; with `attach` = trigger corner.
```rust
right_click_menu::<ContextMenu>("ctx")
    .menu(move |_, _| Some(menu.clone()))
    .trigger(move |_active, _, _| Label::new("right-click me"))
```

---

## 9. Disclosure · Divider · Chip

### `Disclosure` — expand/collapse chevron
```rust
Disclosure::new(id: impl Into<ElementId>, is_open: bool) -> Self
```
Builders: `.on_toggle_expanded(h)`, `.tooltip(...)`, `.opened_icon(IconName)` (default `ChevronDown`), `.closed_icon(IconName)` (default `ChevronRight`), `.disabled(bool)`. Traits: `Toggleable`, `Clickable`, `VisibleOnHover`.
```rust
Disclosure::new("section", self.open).on_toggle_expanded(cx.listener(|this, _, _, cx| {
    this.open = !this.open; cx.notify();
}))
```

### `Divider`
```rust
divider() -> Divider               // horizontal, solid
vertical_divider() -> Divider
Divider::horizontal() | .vertical() | .horizontal_dashed() | .vertical_dashed()
.inset() -> Self                   // adds margin to pull the line in
.color(DividerColor::{Border, BorderFaded, BorderVariant}) -> Self
```
```rust
v_flex().child(Label::new("One")).child(Divider::horizontal()).child(Label::new("Two"))
```

### `Chip` — compact label container
```rust
Chip::new(label: impl Into<SharedString>) -> Self
.label_color(Color) / .label_size(LabelSize) / .icon(IconName) / .icon_color(Color)
.bg_color(Hsla) / .border_color(Hsla) / .height(Pixels) / .truncate() / .tooltip(...)
```
```rust
Chip::new("Beta").label_color(Color::Accent)
Chip::new("New").bg_color(palette::primary(100))
```

---

## 10. Banner · Callout · AlertModal · AnnouncementToast

### `Banner` (ParentElement) — inline status message
```rust
Banner::new() -> Self   // severity default Info
.severity(Severity)   // Info | Success | Warning | Error
.action_slot(element)  // CTA / dismiss
.wrap_content(bool)
```
```rust
Banner::new().severity(Severity::Success)
    .children([Label::new("Saved successfully")])
    .action_slot(Button::new("undo", "Undo").style(ButtonStyle::Subtle))
```

### `Callout` — titled alert with icon + description + actions
```rust
Callout::new() -> Self   // severity Info, border Top
.severity(Severity) / .icon(IconName) / .title(s) / .description(s)
.description_slot(element)  // overrides .description (markdown, etc.)
.actions_slot(element)      // primary CTA
.dismiss_action(element)    // dismiss button
.line_height(Pixels) / .border_position(CalloutBorderPosition::{Top, Bottom})
```
```rust
Callout::new().severity(Severity::Warning).icon(IconName::Warning)
    .title("Subscription expiring")
    .description("Renew now to keep your features.")
    .actions_slot(Button::new("renew", "Renew Now"))
```

### `AlertModal` (ParentElement) — simple confirm dialog
```rust
AlertModal::new(id: impl Into<ElementId>) -> Self
.title(s) / .header(element) / .footer(element) / .primary_action(s)  // default "OK"
.dismiss_label(s)  // default "Cancel"
.width(len)  // default px(440.)
.key_context(s) / .on_action(h) / .track_focus(&fh)
```
```rust
AlertModal::new("leave-call")
    .title("Leave the current call?")
    .child("The current window will be closed.")
    .primary_action("Leave Call")
    .dismiss_label("Cancel")
```

### `AnnouncementToast` — feature announcement
```rust
AnnouncementToast::new() -> Self
.illustration(element) / .heading(s) / .description(s)
.bullet_item(element) / .bullet_items(iter)
.primary_action_label(s) / .primary_on_click(h)
.secondary_action_label(s) / .secondary_on_click(h)
.dismiss_on_click(h)
```

---

## 11. Progress

### `ProgressBar` (determinate only)
```rust
ProgressBar::new(id, value: f32, max_value: f32, cx: &App) -> Self
.value(f32) / .max_value(f32) / .bg_color(Hsla) / .fg_color(Hsla) / .over_color(Hsla)
```
Doc: "A progress bar should not be used to represent indeterminate progress."
```rust
ProgressBar::new("load", 35.0, 100.0, cx)
```

### `CircularProgress`
```rust
CircularProgress::new(value: f32, max_value: f32, size: Pixels, cx: &App) -> Self
.value / .max_value / .size(Pixels) / .stroke_width(Pixels) / .bg_color / .progress_color
```
For an indeterminate spinner, use `Icon::new(IconName::LoadCircle).with_keyed_rotate_animation(id, 2)` (this is what `Button::loading(true)` does internally).

---

## 12. Avatar · Facepile · CountBadge · DiffStat · Vector

### `Avatar`
```rust
Avatar::new(src: impl Into<ImageSource>) -> Self   // path/url
.grayscale(bool) / .border_color(Hsla) / .size(AbsoluteLength) / .indicator(element)
```
```rust
Avatar::new("path/to/img.png").grayscale(true)
Avatar::new(url).indicator(AvatarAudioStatusIndicator::new(AudioStatus::Muted))
```

### `Facepile` (ParentElement)
```rust
Facepile::empty() | Facepile::new(faces: SmallVec<[AnyElement; 2]>)
```
Renders `flex_row_reverse` so the first child is leftmost; children overlap with negative margin. Add `Avatar`s as children.

### `CountBadge`
```rust
CountBadge::new(count: usize) -> Self   // ">99" shows "99+"
```
```rust
IconButton::new("bell", IconName::Bell).indicator(/* badge via overlay */) // or wrap
container().child(CountBadge::new(3))
```

### `DiffStat`
```rust
DiffStat::new(id, added: usize, removed: usize) -> Self
.label_size(LabelSize) / .tooltip(s)   // renders "+N" green / "–M" red
```

### `Vector` / `VectorName` (SVG image, not a raster `Image`)
```rust
Vector::new(vector: VectorName, width: Rems, height: Rems) -> Self
Vector::square(vector, size: Rems)
.color(Color) / .size(Size<Rems>) / .transform(...)
```
`VectorName`: `ZedLogo`, `ZedXCopilot`, `Grid`, `BusinessStamp`, `VipStamp`, `ProTrialStamp`, `ProUserStamp`, `StudentStamp`.

---

## 13. Colors · Severity · Elevation

**For any `bg()`/`border_color()`/fill on a `div()` or new component, use the design tokens —
`semantic::*(cx)` (neutrals) / `palette::role(step)` (accents/status) — not raw `cx.theme()`
calls or `Hsla` literals. Full rules: `references/design-system.md` §1.** The `Color` enum below
remains the idiomatic way to color *text* (`Label::new("x").color(Color::Error)`).

### `Color` (for `Label`/`Icon` text — still current)
```rust
pub enum Color {
    Default,  // foreground text
    Muted, Hidden,   // de-emphasized (Hidden < Muted < Default)
    Accent, Selected,
    Disabled, Placeholder,
    Error, Warning, Success, Info, Hint, Conflict,
    Created, Modified, Deleted, Ignored,
    VersionControlAdded/Conflict/Deleted/Ignored/Modified,
    Debugger, Player(u32),
    Custom(Hsla),  // avoid — prefer palette::role(step) for an arbitrary accent
}
```
Resolve to `Hsla` via `color.color(cx)`.

### `Severity`
```rust
pub enum Severity { Info, Success, Warning, Error }
```
Used by `Banner`, `Callout`, and status indicators. Maps to icon + color automatically.

### `ElevationIndex` (for `.layer(...)` and `elevation_N(cx)`)
Layers: `Background`, `Elevated`, etc. `.elevation_3(cx)` applies the elevated-surface bg + shadow used by modals.
