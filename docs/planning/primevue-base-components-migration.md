# PrimeVue Base Component Migration

Status: Implemented

## Decision

PrimeVue is the implementation source for reusable frontend UI primitives.
Application and feature code should import project-owned `Base*` wrappers instead
of importing PrimeVue components directly. The wrappers can proxy PrimeVue today
and can later own a smaller project API for theming, accessibility, validation,
or behavior changes.

`BaseField` remains local because it owns labels, hints, errors, and ARIA wiring
around controls rather than representing a PrimeVue component.

PrimeVue type-only imports and non-component APIs remain allowed when they are
needed to work with PrimeVue data structures, for example table filter metadata,
tree node types, and filter constants.

## Migration Loop

Each component must be migrated on its own before starting the next component.
Use this loop for future PrimeVue-backed Base components.

1. Migrate the `Base*` component to proxy the matching PrimeVue component.
2. Create or update Histoire stories with common use cases and the extra states
   currently used in Lukas.
3. Test the component with Playwright through interactive behavior, not only
   static unit tests.
4. Run focused unit, type, and lint checks for the touched component and call
   sites.
5. Mark the component complete in this document.

## Component Inventory

| Current component/use | PrimeVue component | Target wrapper | Status | Notes |
| --- | --- | --- | --- | --- |
| `BaseTextInput` | `primevue/inputtext` | `BaseTextInput` | Done | Keeps `v-model`, `invalid`, `describedBy`, and native input attributes. |
| `BaseTextarea` | `primevue/textarea` | `BaseTextarea` | Done | Keeps `v-model`, `invalid`, `describedBy`, and native textarea attributes. |
| `BaseSelect` | `primevue/select` | `BaseSelect` | Done | Changed from native `<option>` slots to typed `options`. |
| `BaseCheckbox` | `primevue/checkbox` | `BaseCheckbox` | Done | Keeps `checked` / `update:checked`; maps to PrimeVue binary mode. |
| `BaseButton` | `primevue/button` | `BaseButton` | Done | Keeps `variant`, `size`, `loading`, `disabled`, and `type`. |
| `BaseAlert` | `primevue/message` | `BaseAlert` | Done | Maps tones to PrimeVue severities. |
| `BaseBadge` | `primevue/badge` | `BaseBadge` | Done | Maps tones to PrimeVue severities. |
| `BaseCard` | `primevue/card` | `BaseCard` | Done | Keeps default and `header` slots. |
| Direct `InputText` | `primevue/inputtext` | `BaseTextInput` | Done | Replaced direct usage in movement table filters. |
| Direct `Select` | `primevue/select` | `BaseSelect` | Done | Replaced direct usage in movement table grouping. |
| Direct `MultiSelect` | `primevue/multiselect` | `BaseMultiSelect` | Done | Added wrapper with typed options. |
| Direct `DataTable` | `primevue/datatable` | `BaseDataTable` | Done | Added thin wrapper and kept table-specific slots. |
| Direct `TreeTable` | `primevue/treetable` | `BaseTreeTable` | Done | Added thin wrapper and kept tree-specific slots. |
| Direct `Column` | `primevue/column` | `BaseColumn` | Done | Exports PrimeVue `Column` directly because table components require structural column children. |

## Public API Changes

- `BaseSelect` will accept `options: Array<{ label: string; value: string }>`
  and will map to PrimeVue with `optionLabel="label"` and
  `optionValue="value"`.
- `BaseMultiSelect` will use the same option shape with
  `modelValue: string[]`.
- `BaseTextInput`, `BaseTextarea`, `BaseCheckbox`, and `BaseButton` keep their
  current public APIs where practical.
- `BaseDataTable` and `BaseTreeTable` are thin component wrappers that pass
  props, events, and slots to PrimeVue.
- `BaseColumn` exports PrimeVue `Column` directly so DataTable and TreeTable can
  detect their structural column children.

## CSS Removal TODO

Remove scoped component styling as each wrapper is migrated. Do not do broad
visual cleanup in this phase.

- [x] `BaseTextInput.vue`: remove `.base-control`.
- [x] `BaseTextarea.vue`: remove `.base-textarea`.
- [x] `BaseSelect.vue`: remove `.base-select`.
- [x] `BaseCheckbox.vue`: remove `.base-checkbox`.
- [x] `BaseButton.vue`: remove `.base-button`, spinner, and keyframes.
- [x] `BaseAlert.vue`: remove `.base-alert`.
- [x] `BaseBadge.vue`: remove `.base-badge`.
- [x] `BaseCard.vue`: remove `.base-card`.
- [x] Keep feature-level CSS and `frontend/app/src/styles/main.css` for later
      consistency work.

## Implementation Order

- [x] Update frontend engineering docs to approve PrimeVue behind Base wrappers.
- [x] Migrate `BaseTextInput`.
- [x] Migrate `BaseTextarea`.
- [x] Migrate `BaseCheckbox`.
- [x] Migrate `BaseSelect` and update all call sites to use options.
- [x] Migrate `BaseButton`.
- [x] Migrate `BaseBadge`.
- [x] Migrate `BaseAlert`.
- [x] Migrate `BaseCard`.
- [x] Add `BaseMultiSelect`.
- [x] Add `BaseDataTable`, `BaseTreeTable`, and `BaseColumn`.
- [x] Replace direct PrimeVue component imports in feature components.
- [x] Run the full frontend unit, typecheck, lint, and Playwright verification
      commands.

## Test Scenarios

- Text input and textarea update `modelValue` and pass native validation
  attributes through to the rendered control.
- Select and multi-select update `modelValue`, show configured options, handle
  disabled and invalid states, and support the app-specific account/category
  option lists.
- Checkbox toggles through PrimeVue binary mode and preserves its project API.
- Button renders slotted content, supports submit buttons, and disables while
  loading.
- Alert, badge, and card render their slots and mapped tones.
- Data table filtering, account multi-select filtering, grouping select, and row
  actions continue to work.
- Budget tree table renders rows and keeps inline editing controls interactive.
