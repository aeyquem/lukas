<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import type { BudgetCategory } from '../api/contracts'
import type { useBudgetAssignments } from '../composables/use-budget-assignments'
import { formatMoney } from '../../../shared/formatting/money'

const props = defineProps<{
  category: BudgetCategory
  closed: boolean
  assignments: ReturnType<typeof useBudgetAssignments>
  disabled: boolean
  pending: boolean
}>()
const key = computed(() => props.assignments.draftKey(props.category.id))
const editing = ref(props.assignments.drafts.value[key.value] !== undefined)
const input = ref<HTMLInputElement | null>(null)
const trigger = ref<HTMLButtonElement | null>(null)
const value = computed(() => props.assignments.drafts.value[key.value] ?? (props.closed ? props.category.assignment_intent ?? props.category.assigned : props.category.assigned))
const reason = computed(() => props.assignments.reasonDrafts.value[key.value] ?? '')

async function beginEdit(): Promise<void> {
  if (props.disabled || props.pending) return
  props.assignments.assignDraft(props.category.id, value.value)
  editing.value = true
  await nextTick()
  input.value?.focus()
  input.value?.select()
}
async function save(): Promise<void> {
  if (props.disabled || props.pending || props.assignments.isStale(props.category.id)) return
  await props.assignments.save(props.category.id)
  // Validation failures, conflicts and unknown outcomes keep the editor and draft.
  if (props.assignments.drafts.value[key.value] !== undefined) return
  editing.value = false
  await nextTick()
  trigger.value?.focus()
}
async function cancel(): Promise<void> {
  if (props.disabled || props.pending) return
  props.assignments.discardDraft(props.category.id)
  editing.value = false
  await nextTick()
  trigger.value?.focus()
}
</script>

<template>
  <div class="assignment-layout">
    <form v-if="editing" class="assignment-form" @submit.prevent="save" @keydown.esc.prevent.stop="cancel">
      <label class="sr-only" :for="`assign-${category.id}`">Asignado para {{ category.name }} en CLP</label>
      <input :id="`assign-${category.id}`" ref="input" class="numeric" :value="value" inputmode="decimal" autocomplete="off" required pattern="[0-9]+(\.[0-9]+)?" :disabled="disabled || pending" @input="assignments.assignDraft(category.id, ($event.target as HTMLInputElement).value)">
      <button class="save-amount" type="submit" :aria-label="`Guardar asignación de ${category.name}`" title="Guardar asignación (Enter)" :disabled="disabled || pending || assignments.isStale(category.id)">
        <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1-2-2h12l4 4v12a2 2 0 0 1-2 2Z"/><path d="M17 21v-8H7v8M7 3v5h8V3"/></svg>
      </button>
      <button class="cancel-amount" type="button" :aria-label="`Cancelar edición de ${category.name}`" title="Cancelar edición (Esc)" :disabled="disabled || pending" @click="cancel">
        <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="m6 6 12 12M18 6 6 18"/></svg>
      </button>
      <label v-if="closed" class="reason-field">Motivo de corrección<textarea :value="reason" required :disabled="disabled || pending" @input="assignments.assignReason(category.id, ($event.target as HTMLTextAreaElement).value)"></textarea></label>
      <template v-if="assignments.isStale(category.id)"><small class="detail">El borrador se conserva. Revisa el objetivo vigente: {{ formatMoney(category.assignment_intent ?? category.assigned, 'CLP') }}</small><button type="button" :disabled="disabled || pending" @click="assignments.reviewCurrent(category.id)">Revisé los valores vigentes de {{ category.name }}</button></template>
    </form>
    <button v-else ref="trigger" type="button" class="assigned-value numeric" :aria-label="`Editar asignación de ${category.name}`" title="Editar asignación" :disabled="disabled || pending" @click="beginEdit">{{ formatMoney(value, 'CLP') }}</button>
  </div>
  <small v-if="closed" class="detail">Original al cierre: {{ formatMoney(category.assigned, 'CLP') }}</small>
  <small v-if="pending" class="detail">Resultado pendiente. Usa Reintentar comando exacto.</small>
</template>

<style scoped>
.assignment-layout { display: grid; grid-template-columns: minmax(0, 9rem) 2.5rem 2.5rem; align-items: center; justify-content: end; gap: .3rem; }
.assignment-form { display: grid; grid-column: 1 / -1; grid-template-columns: minmax(0, 9rem) 2.5rem 2.5rem; gap: .3rem; }
.assignment-form input { width: 100%; min-width: 0; padding: .4rem; text-align: right; }
.save-amount, .cancel-amount { display: inline-flex; align-items: center; justify-content: center; flex: 0 0 2.5rem; width: 2.5rem; height: 42px; min-height: 42px; padding: 0; }
.assigned-value { grid-column: 1; display: block; width: 100%; min-height: 42px; border: 1px solid transparent; border-radius: var(--control-radius); padding: 9px 12px; background: transparent; color: var(--text); text-align: right; font-weight: inherit; }
.assigned-value:hover:not(:disabled) { background: var(--surface-soft); border-color: var(--accent); }
.reason-field, .assignment-form > .detail, .assignment-form > button:not(.save-amount):not(.cancel-amount) { grid-column: 1 / -1; }
.reason-field { display: grid; gap: .3rem; text-align: left; }
.reason-field textarea { max-width: 100%; min-height: 3rem; }
.detail { display: block; margin-top: .3rem; color: var(--muted); font-size: .8rem; font-weight: 400; }
.sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }
</style>
