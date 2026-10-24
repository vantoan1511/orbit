# Animation & Interaction Utilities

This reference covers the PrimeVue directives and Tailwind CSS v4 animation utilities for interactive feedback.

## ⚡ Core Directives

| Directive | Purpose | Key Props / Events |
| :--- | :--- | :--- |
| **`v-ripple`** | Adds interactive ripple feedback to any element. | `--p-ripple-background` (CSS Var) |
| **`v-animateonscroll`** | Triggers animations as elements enter/leave the viewport. | `enterClass`, `leaveClass` |
| **`v-styleclass`** | Declarative class toggling for transitions or UI states. | `selector`, `enterActiveClass`, `hideOnOutsideClick` |
| **`v-focustrap`** | Keeps keyboard focus within a specific container. | Standard directive |

---

## 🎬 Tailwind v4 Transitions

The PrimeVue Tailwind plugin exposes utility classes for building enter/leave transitions:

### 1. Keyframe Modifiers
Combine `animate-enter` or `animate-leave` with these modifiers:
- `fade-in-{value}` / `fade-out-{value}`
- `zoom-in-{value}` / `zoom-out-{value}`
- `slide-in-from-{dir}-{value}` / `slide-out-to-{dir}-{value}`
- `spin-in-{value}` / `spin-out-{value}`

### 2. Transition Control
- **Duration**: `animate-duration-[ms]`
- **Delay**: `animate-delay-[ms]`
- **Iterations**: `animate-infinite`, `animate-once`
- **Easing**: `animate-ease-in-out`, `animate-ease-linear`

---

## ✨ Best Practices

1. **Accessibility First**: Always ensure content is visible even if animations fail. Use `aria-hidden="true"` on purely visual ripples.
2. **Performance**: Avoid animating layout-heavy properties like `width`, `height`, or `margin`. Prefer `transform` (translate, scale) and `opacity`.
3. **Subtlety**: Use short durations (e.g., `animate-duration-200`) for interactive feedback to keep the UI feeling snappy.


