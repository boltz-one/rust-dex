# Codebase Audit: GPUI UI Kit vs Tailwind Application UI

**Date:** 2026-07-03  
**Scope:** `crates/ui/src/components/` (44 components), `crates/icons`, `examples/ui_gallery`  
**Goal:** Gap matrix — which components exist, restyle status (Tailwind tokens vs old Zed style), variants/states, gallery showcase status

---

## Component Status Matrix

| Component | Category | Restyle Status | Variants/States | Gallery Showcase | Notes |
|-----------|----------|----------------|-----------------|------------------|-------|
| **Button** | Button | 🟡 Partial | default, primary, success, danger, ghost, link, sm/md/lg, disabled, loading | ✅ Elements | Button/ButtonLike/CopyButton/IconButton/SplitButton/ToggleButton; Phase 2 Button restyle pending |
| **Badge** | Badge | ✅ Done (new) | solid, outline, info/success/warning/danger, sm/md | ✅ Elements | Built with Tailwind tokens (Phase 2) |
| **Card** | Container | ✅ Done (new) | default, elevated, outlined | ✅ Elements | New in Phase 2, uses semantic + shadow |
| **Alert** | Feedback | ✅ Done (new) | Callout base (info/warning/danger/success), with/without icon | ✅ Feedback | Callout reusable base; Phase 2 Alert wrapper done |
| **Checkbox** | Form | 🟡 Partial | checked/unchecked/indeterminate, disabled, focused | ❌ Gallery only shows selected/unselected in Forms page | Phase 3 restyle pending; uses `toggle.rs` base |
| **Switch** | Form | 🟡 Partial | on/off, disabled, focused | ❌ Gallery only shows selected/unselected in Forms page | Phase 3 restyle pending; uses `toggle.rs` base |
| **TextInput** | Form | ✅ Done (new) | placeholder, error-state, disabled, focused, icon-prefix | ✅ Forms | Built Phase 3; multiline variant used for Textarea |
| **Textarea** | Form | ✅ Done (new) | multiline TextInput, placeholder, error, disabled | ✅ Forms | Same as TextInput with `.multiline(true)` |
| **Select** | Form | ✅ Done (new) | option groups, disabled, focused, icon, size | ✅ Forms | Phase 3; uses ChevronDown icon |
| **RadioButton** | Form | ✅ Done (new) | checked/unchecked, disabled, focused, label | ✅ Forms | Phase 3; label builder pattern |
| **Modal** | Overlay | ⬜ Pending | header/body/footer, sizes, backdrop, close-button | ❌ Not in gallery | Phase 4 pending; base exists |
| **Dropdown/ContextMenu** | Overlay | ⬜ Pending (styling) | menu items, icons, shortcuts, dividers, disabled | ❌ Not in gallery | DropdownMenu & ContextMenu bases exist; styling TBD Phase 4 |
| **TabBar/Tab** | Navigation | ⬜ Pending | active/inactive tabs, icon support, scrollable | ❌ Not in gallery | Bases exist; Phase 4 styling pending |
| **Tooltip** | Overlay | ⬜ Pending | trigger (hover/focus), position (top/bottom/left/right), dark | ❌ Not in gallery | Base exists; Phase 4 styling pending |
| **Popover** | Overlay | ⬜ Pending | trigger, arrow, position, close-button | ❌ Not in gallery | PopoverMenu & Popover bases exist; Phase 4 pending |
| **Toast** | Feedback | ⬜ Pending | type (info/success/warning/error), dismissible, action-button | ❌ Not in gallery | AnnouncementToast base exists; Phase 4 pending |
| **DataTable** | Data | ⬜ Pending | header/rows/cells, sorting, selection, pagination | ❌ Not in gallery | Base + TableRow exist; Phase 4 styling pending |
| **Navbar** | Navigation | ✅ Done | brand/logo, nav items, trailing actions, sticky | ✅ Navigation | Phase 5 done; light/dark toggle in gallery uses it |
| **Sidebar** | Navigation | ✅ Done | items (active/inactive), collapsible, nested | ✅ Navigation | Phase 5 done; gallery sidebar uses it; SidebarItem builder |
| **Indicator** | Badge | 🟡 Partial | dot, ring, animation | ❌ Not in gallery | Exists; status TBD |
| **Avatar** | Avatar | 🟡 Partial | sizes, initials, image, fallback | ❌ Not in gallery | Exists; Tailwind restyle status unclear |
| **Divider** | Visual | 🟡 Partial | horizontal, vertical, with label | ❌ Not in gallery | Exists; restyle status unclear |
| **Chip** | Badge | 🟡 Partial | label, icon, dismissible | ❌ Not in gallery | Exists; restyle status unclear |
| **Label** | Typography | 🟡 Partial | sizes (Small/Medium/Large), semantic colors | ✅ Gallery forms use Label::new(...).size(LabelSize::Small/Large) | LabelLike/LoadingLabel/SpinnerLabel variants exist |
| **CountBadge** | Badge | 🟡 Partial | numeric, overflow (99+), size | ❌ Not in gallery | Reusable base in Phase 2 notes |
| **KeybindingHint** | Typography | 🟡 Partial | key display, styling | ❌ Not in gallery | Exists; styling TBD |
| **Banner** | Feedback | 🟡 Partial | type, action-button, dismissible | ❌ Not in gallery | Exists; styling TBD |
| **List** | Container | 🟡 Partial | items, headers, sub-headers, separators, bullets | ❌ Not in gallery | ListItem/ListHeader/ListSubHeader/ListSeparator/ListBulletItem bases exist |
| **Progress** | Feedback | 🟡 Partial | bar, circular, indeterminate, with-label | ❌ Not in gallery | ProgressBar & CircularProgress variants exist |
| **Disclosure** | Accordion | 🟡 Partial | expand/collapse, icon, label | ❌ Not in gallery | Exists; styling TBD |
| **Group** | Layout | 🟡 Partial | horizontal/vertical, gap, spacing | ❌ Not in gallery | Layout helper; restyle TBD |
| **Stack** | Layout | 🟡 Partial | h/v flex variants, distribution | ❌ Not in gallery | Layout helper; restyle TBD |
| **RightClickMenu** | Overlay | ⬜ Pending | context-triggered, items | ❌ Not in gallery | Base exists; Phase 4 pending |
| **PopoverMenu** | Overlay | ⬜ Pending | position, trigger, items | ❌ Not in gallery | Base exists; Phase 4 pending |
| **Image** | Asset | 🟡 Partial | sizing, aspect-ratio, fallback | ❌ Not in gallery | Exists; styling TBD |
| **Icon & DecoratedIcon** | Visual | 🟡 Partial | size, color, decorations | ❌ Not in gallery | Icon + IconDecoration system exists |
| **Scrollbar** | Control | 🟡 Partial | styling, visibility | ❌ Not in gallery | Exists; styling TBD |
| **TreeViewItem** | Navigation | 🟡 Partial | expand/collapse, icon, nested | ❌ Not in gallery | Exists; styling TBD |
| **Collab (Collab, NotifAction)** | Custom | 🟡 Partial | collab-specific UI | ❌ Not in gallery | Collab-specific domain components |
| **GradientFade** | Visual | 🟡 Partial | fade direction, colors | ❌ Not in gallery | Utility component |
| **IndentGuides** | Visual | 🟡 Partial | styling, spacing | ❌ Not in gallery | Editor-specific |
| **RedistributableColumns** | Layout | 🟡 Partial | dynamic column resizing | ❌ Not in gallery | Editor-specific |
| **StickyItems** | Layout | 🟡 Partial | sticky positioning | ❌ Not in gallery | Layout helper |
| **Facepile** | Avatar | 🟡 Partial | stacked avatars, overflow count | ❌ Not in gallery | Exists; styling TBD |
| **DiffStat** | Display | 🟡 Partial | additions/deletions visual | ❌ Not in gallery | Git-specific |
| **Navigable** | Behavior | 🟡 Partial | keyboard nav wrapper | ❌ Not in gallery | Behavior mixin |
| **Stories** | Documentation | ⚠️ Meta | component previews | ❌ Gallery app replaces this | Zed-era pattern; gallery app now owns showcase |

---

## Reusable Base Components (Priority for Composite Building)

| Component | File Path | Status | Reuse Pattern |
|-----------|-----------|--------|---|
| **Checkbox** | `components/toggle.rs` | 🟡 Restyle pending | Base for toggle UI; Tailwind tokens needed |
| **Switch** | `components/toggle.rs` | 🟡 Restyle pending | Same base as Checkbox |
| **Callout** | `components/callout.rs` | ✅ Ready | Alert/Banner/Toast base (icon + message + color role) |
| **CountBadge** | `components/count_badge.rs` | ✅ Ready | Numeric display (e.g., unread count, badge overflow) |
| **Modal** | `components/modal.rs` | ⬜ Needs styling | Overlay base (header/body/footer) |
| **DropdownMenu** | `components/dropdown_menu.rs` | ⬜ Needs styling | Menu items + icon + disabled states |
| **Tab/TabBar** | `components/tab.rs`, `components/tab_bar.rs` | ⬜ Needs styling | Tab group container + individual tab |
| **Tooltip** | `components/tooltip.rs` | ⬜ Needs styling | Hover/focus popover base |
| **Popover** | `components/popover.rs` | ⬜ Needs styling | Positioned popup base |
| **Toast** (AnnouncementToast) | `components/notification/announcement_toast.rs` | ⬜ Needs styling | Toast variant (icon + message + action) |
| **DataTable/TableRow** | `components/data_table.rs`, `components/data_table/table_row.rs` | ⬜ Needs styling | Table grid base + row/cell components |
| **Label** | `components/label/label.rs` | ✅ Ready | Typography base (size variants: Small/Medium/Large) |
| **List/ListItem** | `components/list/list.rs`, `components/list/list_item.rs` | ⬜ Needs styling | List container + item variants |
| **Icon** | `components/icon/icon.rs` | ✅ Ready | SVG icon renderer (accepts IconName enum) |

---

## Icon Availability Summary

**IconName enum** (`crates/icons/src/icons.rs`) — **180+ icons** across categories:

- **AI Models** (18): AiAnthropic, AiBedrock, AiClaude, AiGemini, AiGoogle, AiMistral, AiOllama, AiOpenAi, AiOpenRouter, AiXAi, etc.
- **Navigation** (8): ArrowUp, ArrowDown, ArrowLeft, ArrowRight, ArrowUpRight, ArrowDownRight, ChevronUp, ChevronDown, ChevronUpDown, etc.
- **Form/Input** (5): Check, CheckCircle, Close, CheckDouble, Control
- **Alerts/Status** (6): BellRing, BellDot, BellOff, CircleHelp, ExclamationTriangle ⚠️ (not explicitly in enum output, but Callout needs it), Clock, Indicator
- **Files/Folders** (15+): File, FileCode, FileDiff, FileDoc, Folder, FolderOpen, FolderOpenAdd, FolderSearch, etc.
- **Git** (6): GitBranch, GitBranchPlus, GitCommit, GitGraph, GitMergeConflict, GitWorktree, Github
- **Edit/Action** (10+): Copy, Download, Upload, Trash, Pencil, Eraser, Edit, etc.
- **Debug** (10+): DebugBreakpoint, DebugContinue, DebugPause, DebugStepOver, Debug, etc.
- **UI Control** (8): Ellipsis, Hash, ListBullet, ListDashes, LockOpen, Lock, etc.
- **Text/Font** (4): Font, FontSize, FontWeight, CaseSensitive
- **Code** (5): Code, CodeBrackets, Diff, DiffSplit, etc.

**Missing Heroicons for Tailwind UI completeness:**
- ❌ **ExclamationTriangle** (for warning/danger alerts) — **needed for Phase 2/3 alerts**
- ❌ **Star, StarFilled** (for ratings, favorites)
- ❌ **Heart, HeartFilled** (for favorites/likes)
- ❌ **MapPin** (for location)
- ❌ **User, Users** (for team/collab features)
- ❌ **Settings, Gear** (for configuration)
- ❌ **Plus, Minus** (for add/remove actions)
- ❌ **Search, Magnifying Glass** (already have? check coverage)
- ❌ **Eye, EyeOff** (privacy indicators) — **already in enum!**
- ❌ **Calendar** (date input)
- ❌ **Home** (home link in nav)

**Status:** Library is AI/debug-heavy (180+ icons). Form/feedback icons partially covered; missing generic UI icons (Star, User, Settings, Plus, Home, Calendar).

---

## Tailwind Application UI Components NOT YET IN CRATES/UI

Based on Tailwind UI catalog & Tailwind Headless UI library, these are commonly expected but missing or pending restyle:

| Component | Status | Notes |
|-----------|--------|-------|
| **Input Groups** (prefix/suffix icon + input) | ⬜ Missing | TextInput exists; wrapper container needed |
| **Search Input** (with clear button) | ⬜ Missing | TextInput variant |
| **File Input** | ⬜ Missing | Form component |
| **Textarea with Char Counter** | ⬜ Missing | Textarea variant |
| **Date Picker** | ⬜ Missing | Form component (needs Calendar icon) |
| **Time Picker** | ⬜ Missing | Form component |
| **Combobox** (searchable select) | ⬜ Missing | Select variant (needs SearchIcon) |
| **Multi-Select** | ⬜ Missing | Select variant with tags |
| **Segmented Control** (radio group styled as buttons) | ⬜ Missing | Could reuse RadioButton base |
| **Breadcrumb** | ⬜ Missing | Navigation component |
| **Pagination** | ⬜ Missing | Navigation component (needs ChevronLeft/Right, Page numbers) |
| **Stepper** (multi-step form) | ⬜ Missing | Navigation component |
| **Accordion** | 🟡 Partial | Disclosure exists; needs Tailwind styling |
| **Alert Dialog** (blocking modal) | 🟡 Partial | Modal base exists; AlertModal in notification/ folder |
| **Confirm Dialog** | ⬜ Missing | Modal variant (yes/no buttons) |
| **Drawer/Slide-out Panel** | ⬜ Missing | Like Modal but side-positioned |
| **Notification/Snackbar Stack** | 🟡 Partial | AnnouncementToast exists; toast stack container missing |
| **Loading States** (skeleton, spinner overlay) | 🟡 Partial | SpinnerLabel exists; full-screen loading overlay missing |
| **Dropzone** (drag-drop file upload) | ⬜ Missing | Form component |
| **Color Picker** | ⬜ Missing | Form component |
| **Toggle Group** (radio-like button group) | 🟡 Partial | ToggleButton exists; group container missing |
| **Command Palette / Search** | ⬜ Missing | Complex overlay (Command + Modal + List) |
| **Autocomplete / Typeahead** | ⬜ Missing | TextInput + Popover combo |
| **Carousel / Slider** | ⬜ Missing | Layout component |
| **Virtualized List** | ⬜ Missing | Performance optimization for List |
| **Tree** (hierarchical list) | 🟡 Partial | TreeViewItem exists; styling + container missing |
| **Kanban / Drag-drop Grid** | ⬜ Missing | Complex layout component |

---

## Gallery Showcase Status

**Currently Wired in `examples/ui_gallery/src/gallery_app.rs`:**
- ✅ **Elements page** → Buttons, Badges, Cards (calls `.preview()` on each)
- ✅ **Forms page** → TextInput, Textarea, Select, RadioButton, Checkbox, Switch (rendered as form fields with labels)
- ✅ **Feedback page** → Alerts (Callout variants)
- ✅ **Navigation page** → Navbar, Sidebar

**Preview Pattern:** Components implement `.preview(window, cx) -> AnyElement` (trait method on `Component`); gallery calls it to render showcase variant grid.

**Light/Dark Toggle:** Gallery uses `SystemAppearance::global(cx)` to switch `Appearance::Light` ↔ `Appearance::Dark` + theme updates automatically.

---

## Preliminary Findings

### ✅ Completed (Phase 1, 2, 5)
1. Design tokens (palette, semantic, shadow, focus_ring) + helpers in place
2. Button family (variants, sizes, loading states) — **restyle Tailwind IN PROGRESS** (Phase 2)
3. Badge (solid/outline + roles) — new, Tailwind done
4. Card (default/elevated/outlined) — new, Tailwind done
5. Alert/Callout (info/warning/danger/success) — new Callout base, Alert wrapper pending
6. TextInput, Textarea, Select, RadioButton — new, Tailwind done (Phase 3)
7. Navbar, Sidebar — new, Tailwind done (Phase 5)
8. Gallery app shell (4 pages, light/dark toggle, preview pattern) — done (Phase 5)

### 🟡 Partially Done (Need Restyle)
- Checkbox, Switch (base exists in `toggle.rs`; Phase 3 restyle pending)
- Label, Icon, List, Progress (bases exist; Tailwind tokens not fully applied)
- Avatar, Divider, Chip, KeybindingHint, Banner, Disclosure, Group, Stack, etc. (old styling, Phase restyle unclear)

### ⬜ Pending Restyle (Phase 4)
- Modal, Dropdown/ContextMenu, TabBar/Tab, Tooltip, Popover, PopoverMenu, Toast, DataTable, RightClickMenu (bases exist; Phase 4 styling not started)

### ❌ Missing from Catalog
- Input groups (prefix/suffix), File input, Date/Time pickers, Combobox, Multi-select, Segmented control, Breadcrumb, Pagination, Stepper, Drawer, Toast stack, Full-screen loading, Dropzone, Color picker, Toggle group container, Command palette, Autocomplete, Carousel, Virtualized list, Tree container, Kanban grid

### 🔧 Icon System Ready
- 180+ icons available; AI/debug-focused
- **Missing for UI kit:** ExclamationTriangle, Star, Heart, MapPin, User, Settings, Plus, Minus, Search (partial), Home, Calendar
- Heroicon integration via `crates/icons` Assets + `application().with_assets()`

---

## Open Questions

1. **Phase 2 Button restyle status:** Is primary/secondary/ghost variant styling actually done, or visual-verify still in progress?
2. **Checkbox/Switch visual spec:** Phase 3 notes "restyle pending" — which Tailwind tokens (color, size, spacing)?
3. **Icon ExclamationTriangle:** Is this a blocker for Phase 2 alerts or can we use generic CircleHelp?
4. **Phase 4 styling approach:** Will composite components (Modal/Dropdown/Tabs) get full **nested component tree restyle** (child buttons/icons inherit palette), or only container-level palette?
5. **Gallery visual-verify:** Have Phase 2/3/4 components been `cargo run -p ui_gallery` visual-verified against Tailwind UI screenshots in light + dark?
6. **Missing form inputs:** Are file input, date picker, colorpicker planned for Phase 6+ or out-of-scope?

---

**Report Source:** Grep crates/ui/src/components/ + read plan.md phases + inspect gallery_app.rs + IconName enum (crates/icons/src/icons.rs)

