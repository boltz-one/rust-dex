# Tailwind UI Specification for GPUI Native UI Kit

**Date:** 2026-07-02  
**Scope:** Design tokens + Application UI component catalog mapped for GPUI px()/hsla() implementation

---

## 1. Design Tokens

### 1.1 Color Palette (Hex Values)

**Neutral Scale (Slate/Gray/Zinc)**

| Shade | Slate | Gray | Zinc |
|-------|-------|------|------|
| 50 | #f8fafc | #f9fafb | #fafafa |
| 100 | #f1f5f9 | #f3f4f6 | #f4f4f5 |
| 200 | #e2e8f0 | #e5e7eb | #e4e4e7 |
| 300 | #cbd5e1 | #d1d5db | #d4d4d8 |
| 400 | #94a3b8 | #9ca3af | #a1a1a6 |
| 500 | #64748b | #6b7280 | #71717a |
| 600 | #475569 | #4b5563 | #52525b |
| 700 | #334155 | #374151 | #3f3f46 |
| 800 | #1e293b | #1f2937 | #27272a |
| 900 | #0f172a | #111827 | #18181b |
| 950 | #020617 | #030712 | #09090b |

**Semantic Colors** (state + accent)

| Intent | Shade 500 | Shade 600 | Shade 700 |
|--------|-----------|-----------|-----------|
| Blue (primary) | #3b82f6 | #2563eb | #1d4ed8 |
| Indigo (accent) | #6366f1 | #4f46e5 | #4338ca |
| Red (error) | #ef4444 | #dc2626 | #b91c1c |
| Green (success) | #22c55e | #16a34a | #15803d |
| Amber (warning) | #f59e0b | #d97706 | #b45309 |

**Sources:** [Tailwind Colors](https://tailwindcss.com/docs/colors), [Tailwind v3 Customizing Colors](https://v3.tailwindcss.com/docs/customizing-colors)

### 1.2 Spacing Scale (rem → px)

| Token | rem | px |
|-------|-----|-----|
| px | 0.0625 | 1 |
| 0.5 | 0.125 | 2 |
| 1 | 0.25 | 4 |
| 1.5 | 0.375 | 6 |
| 2 | 0.5 | 8 |
| 2.5 | 0.625 | 10 |
| 3 | 0.75 | 12 |
| 3.5 | 0.875 | 14 |
| 4 | 1 | 16 |
| 6 | 1.5 | 24 |
| 8 | 2 | 32 |
| 10 | 2.5 | 40 |
| 12 | 3 | 48 |

**Mapping:** 1 rem = 4px (Tailwind default). Use `px(4)`, `px(8)`, `px(12)`, etc. in GPUI.

**Source:** [Tailwind Spacing](https://tailwindcss.com/docs/spacing)

### 1.3 Border Radius (px)

| Token | Value | GPUI |
|-------|-------|------|
| none | 0 | px(0) |
| xs | 2 | px(2) |
| sm | 4 | px(4) |
| md | 6 | px(6) |
| lg | 8 | px(8) |
| xl | 12 | px(12) |
| 2xl | 16 | px(16) |
| 3xl | 24 | px(24) |
| full | ∞ | 9999px |

**Source:** [Tailwind Border Radius](https://tailwindcss.com/docs/border-radius)

### 1.4 Typography

| Token | Size (px) | Weight | Line-Height | Use |
|-------|-----------|--------|-------------|-----|
| xs | 12 | 400/500 | 1rem | Labels, badges |
| sm | 14 | 400/500 | 1.25rem | Secondary text |
| base | 16 | 400/500 | 1.5rem | Body text (default) |
| lg | 18 | 500 | 1.75rem | Subheadings |
| xl | 20 | 600 | 1.75rem | Headings |
| 2xl | 24 | 700 | 2rem | Section headings |
| 3xl | 30 | 700 | 2.25rem | Page headings |

**Font Family:** Inter (system fallback: -apple-system, BlinkMacSystemFont, sans-serif)  
**Source:** [Tailwind Typography](https://v3.tailwindcss.com/docs/font-size)

### 1.5 Shadows

| Token | Box-Shadow | Use |
|-------|-----------|-----|
| sm | 0 1px 2px 0 rgba(0,0,0,0.05) | Subtle elevation |
| base/md | 0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -1px rgba(0,0,0,0.06) | Default elevation |
| lg | 0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -2px rgba(0,0,0,0.05) | Modal/popover |
| xl | 0 20px 25px -5px rgba(0,0,0,0.1), 0 10px 10px -5px rgba(0,0,0,0.04) | Floating UI |

### 1.6 Focus Ring (Accessibility)

- **Focus ring:** `ring-2` (2px) + `ring-offset-2` (2px gap)  
- **Ring color:** Blue-500 (#3b82f6) for interactive elements  
- **Map to GPUI:** `outline(1px, gpui_color::blue().opacity(0.5))` + offset

---

## 2. Application UI Component Catalog

### 2.1 Buttons

**Variants:** primary, secondary, soft, white, outline, ghost, danger  
**Sizes:** xs (padding 6px 12px), sm (8px 14px), md (10px 16px), lg (12px 18px), xl (14px 20px)  
**Common:** 
- Primary: bg-blue-600 hover:bg-blue-700, text white, rounded-md, shadow-sm
- Secondary: bg-white border border-gray-300, text gray-900, hover:bg-gray-50
- Soft: bg-blue-50, text blue-700, hover:bg-blue-100
- Ghost: transparent, text blue-600, hover:bg-blue-50
- Icon variants: left/right icon + text, icon-only

**State:** disabled (opacity-50, cursor-not-allowed), loading (spinner), active (ring-2 ring-blue-500)

**Source:** [Tailwind UI Buttons](https://tailwindui.com/components/application-ui/elements/buttons)

### 2.2 Form Components

**Text Input:** border border-gray-300, rounded-md, px-3 py-2, focus:ring-2 ring-blue-500, placeholder-gray-400  
**Textarea:** Same styling, min-height 6rem, resize vertical  
**Select/Dropdown:** border-gray-300, arrow icon (custom), hover:border-gray-400  
**Checkbox:** 4×4 size, border border-gray-300, checked:bg-blue-600, focus:ring-2  
**Radio Button:** 4×4, border border-gray-300, checked:border-blue-600 checked:bg-blue-600  
**Toggle/Switch:** width 44px, height 24px, bg-gray-200 (off) / bg-blue-600 (on), smooth transition  
**Label:** text-sm font-medium text-gray-700, mb-1  
**Help Text:** text-xs text-gray-500, mt-1  
**Error State:** border-red-500, ring-red-500, text-red-600 (message)

**Source:** [Tailwind UI Forms](https://tailwindui.com/components/application-ui/forms/form-layouts), [Toggles](https://tailwindui.com/components/application-ui/forms/toggles)

### 2.3 Badges

**Types:** solid, soft, outline, dot  
**Colors:** gray/blue/red/green/amber (primary color 500/600)  
**Solid:** bg-{color}-100, text {color}-800, rounded-full, px-2 py-1, text-xs font-medium  
**Soft:** bg-{color}-50, text {color}-700  
**Outline:** border border-{color}-300, text {color}-700, bg-white  
**Dot:** Colored dot (6px circle) + text, gap-1.5

### 2.4 Cards

**Structure:** bg-white, border border-gray-200, rounded-lg, shadow-sm, p-6 (padding)  
**Variants:** elevated (shadow-md), bordered (no shadow), flat (no border, no shadow)  
**Sections:** header (flex justify-between), body, footer (flex gap-3)  
**Hover:** shadow-md transition (optional interactive state)

### 2.5 Alerts

**Types:** info, success, warning, error  
**Layout:** flex gap-3, left icon (16px), text, right dismiss icon  
**Colors (border left + bg + text):**
- Info: border-blue-200, bg-blue-50, text-blue-800
- Success: border-green-200, bg-green-50, text-green-800
- Warning: border-amber-200, bg-amber-50, text-amber-800
- Error: border-red-200, bg-red-50, text-red-800
**Size:** px-4 py-3, border-l-4 (left accent)

### 2.6 Tables

**Header:** bg-gray-50, border-b border-gray-200, font-semibold text-sm  
**Rows:** border-b border-gray-200, py-3 px-4, text-sm  
**Striped:** alternate bg-white / bg-gray-50  
**Hover:** row hover:bg-gray-100 (optional)  
**Sorting:** clickable header with up/down arrow (chevron)  
**Pagination:** flex justify-center, gap-1, button variants (disabled, active)

**Source:** [Tailwind UI Tables](https://tailwindui.com/components/application-ui/lists/tables)

### 2.7 Modals/Dialogs

**Overlay:** fixed inset-0, bg-black/50 (backdrop), z-50  
**Container:** bg-white, rounded-lg, shadow-xl, max-width 448px (sm) / 560px (md) / 672px (lg)  
**Header:** border-b border-gray-200, flex justify-between, close icon (top-right)  
**Body:** p-6, text-base  
**Footer:** border-t border-gray-200, flex gap-3 justify-end, p-4 (buttons)

**Source:** [Tailwind UI Modals](https://tailwindui.com/components/application-ui/overlays/modals)

### 2.8 Dropdowns/Menus

**Trigger:** button (see Buttons section)  
**Menu:** bg-white, border border-gray-200, rounded-md, shadow-lg, z-40  
**Items:** px-4 py-2, text-sm, hover:bg-gray-100, cursor-pointer  
**Separator:** border-t border-gray-200, my-1  
**Disabled item:** text-gray-400, cursor-not-allowed

### 2.9 Tabs

**Underline style:** flex gap-8, border-b border-gray-200, text-sm font-medium  
- Active: text-blue-600, border-b-2 border-blue-600, py-4
- Inactive: text-gray-500, hover:text-gray-700

**Pills style:** flex gap-2, bg-gray-100 rounded-lg p-1  
- Active: bg-white, text-gray-900, shadow-sm
- Inactive: text-gray-600, hover:bg-gray-50

### 2.10 Navbar/Sidebar

**Navbar:** bg-white, border-b border-gray-200, sticky top-0, flex items-center px-6 py-4, shadow-sm  
**Sidebar:** bg-gray-900 (dark) or bg-white (light), fixed left-0 top-0, width 256px, h-screen, border-r  
**Nav links:** px-4 py-2, text-sm, rounded-md, hover:bg-gray-100 (light) / hover:bg-gray-800 (dark)  
**Active link:** bg-gray-100, text-gray-900 (light) / bg-gray-800, text-white (dark)

### 2.11 Avatars

**Sizes:** xs (24px), sm (32px), md (40px), lg (48px), xl (56px), 2xl (64px)  
**Styles:** image, initials (bg-{color}-600, text white, font-semibold, centered), icon placeholder  
**Border:** optional border-2 border-white (grouped avatars)  
**Status indicator:** small circle (green/red) positioned top-right, 6px diameter

### 2.12 Tooltips

**Content:** bg-gray-900, text-white, text-xs, px-2 py-1, rounded-md, shadow-lg  
**Arrow:** small triangle, pointing to trigger  
**Placement:** top/bottom/left/right, 4px gap from trigger  
**Animation:** fade in/out, no scale (subtle)

### 2.13 Notifications/Toasts

**Layout:** fixed bottom-right (or top-right), bg-white, border border-gray-200, rounded-lg, shadow-lg, p-4, max-width 384px  
**Content:** icon (left), title + message, close button (right)  
**Color variants:** apply border + icon color per type (success/error/warning/info)  
**Auto-dismiss:** 5s timeout (configurable)  
**Stack:** gap-3 between multiple toasts, max 3 visible

---

## 3. Phase Prioritization for GPUI Build

1. **Phase 1 (Core):** Buttons (6 variants), Text Input, Labels, Badges, Alert  
2. **Phase 2 (Forms):** Textarea, Select, Checkbox, Radio, Toggle/Switch, Help text, Error state  
3. **Phase 3 (Data):** Card, Table (striped + hover), Badge variants  
4. **Phase 4 (Overlays):** Modal, Dropdown, Tooltip, Toast  
5. **Phase 5 (Navigation):** Navbar, Sidebar, Tabs (underline + pills)  
6. **Phase 6 (Complex):** Avatar groups, advanced table features, customizable card layouts

---

## Open Questions

- Tailwind v4 (OKLCH colors): exact GPUI color space mapping (sRGB ↔ OKLCH conversion needed for precise token values)
- Focus ring accessibility: does GPUI outline() support offset? Alternative: nested outer ring via separate view
- Interactive state animations: duration/easing (0.15s cubic-bezier default in Tailwind)
- Responsive breakpoints (sm/md/lg/xl): are these needed for native desktop? Or only fixed layouts?

---

## Trade-Offs

- **Hex vs. OKLCH:** Using hex values (extracted above) avoids color space conversion; OKLCH offers perceptual uniformity but requires runtime conversion
- **Icon integration:** Tailwind UI components assume icon libraries (Heroicons); GPUI will need embedded SVG or icon system  
- **CSS shadows to GPUI:** Direct translate of box-shadow CSS values; verify GPUI shadow render quality vs. CSS
- **Responsive vs. fixed:** Tailwind UI examples are responsive; native desktop can use fixed layouts initially, add adaptive sizes later
