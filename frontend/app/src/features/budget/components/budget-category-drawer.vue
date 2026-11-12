<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, useTemplateRef, watch } from 'vue'
import type { BudgetGroup, CategoryDraft, ExpenseCategory } from '../api/contracts'
const props = defineProps<{
  open: boolean; category: ExpenseCategory | null; draft: Pick<CategoryDraft, 'name' | 'group_id'> | null
  groups: readonly BudgetGroup[]; disabled: boolean; error: string; stale: boolean; deleteConfirmed: boolean
}>()
const emit = defineEmits<{
  close: []; save: []; archive: []; remove: []; review: []
  nameChange: [name: string]; groupChange: [groupId: string]; deleteConfirmation: [confirmed: boolean]
}>()
const dialog = useTemplateRef<HTMLDialogElement>('dialog')
const nameInput = useTemplateRef<HTMLInputElement>('nameInput')
const availableGroups = computed(() => props.groups.filter((group) => !group.archived || group.id === props.draft?.group_id))
let opener: HTMLElement | null = null
watch(() => props.open, async (open) => {
  await nextTick()
  if (open && dialog.value && !dialog.value.open) {
    opener = document.activeElement instanceof HTMLElement ? document.activeElement : null
    dialog.value.showModal()
    nameInput.value?.focus()
  } else if (!open && dialog.value?.open) dialog.value.close()
}, { immediate: true })
function closed(): void {
  emit('close')
  void nextTick(() => {
    if (opener?.isConnected && opener.getClientRects().length) opener.focus()
    else document.querySelector<HTMLElement>('main button:not([disabled])')?.focus()
  })
}
onBeforeUnmount(() => { if (dialog.value?.open) dialog.value.close() })
</script>

<template>
  <Teleport to="body">
    <dialog ref="dialog" class="category-drawer" aria-labelledby="category-drawer-title" @close="closed" @cancel.prevent="emit('close')">
      <header class="drawer-heading"><h2 id="category-drawer-title">{{ category ? 'Detalles de categoría' : 'Nueva categoría' }}</h2><button type="button" @click="emit('close')">Cerrar detalles</button></header>
      <p v-if="category?.archived" class="drawer-note">Archivada. Las transacciones, asignaciones y cierres conservan su historial.</p>
      <p v-if="error" class="drawer-error" role="alert">{{ error }}</p>
      <p v-if="stale" class="drawer-note">La categoría cambió desde que abriste el borrador. Compara los datos actuales: {{ category?.name }} · {{ groups.find((group) => group.id === category?.group_id)?.name }}.<button type="button" :disabled="disabled" @click="emit('review')">Revisar versión actual</button></p>
      <form v-if="draft" class="drawer-form" @submit.prevent="emit('save')">
        <label class="drawer-field">Nombre de categoría<input ref="nameInput" :value="draft.name" :disabled="disabled" required maxlength="80" autocomplete="off" @input="emit('nameChange', ($event.target as HTMLInputElement).value)"></label>
        <label class="drawer-field">Grupo de categoría<select :value="draft.group_id ?? ''" :disabled="disabled" required @change="emit('groupChange', ($event.target as HTMLSelectElement).value)"><option value="" disabled>Selecciona un grupo</option><option v-for="group in availableGroups" :key="group.id" :value="group.id" :disabled="group.archived">{{ group.name }}{{ group.archived ? ' (archivado)' : '' }}</option></select></label>
        <button type="submit" :disabled="disabled || stale">Guardar categoría</button>
      </form>
      <section v-if="category" class="drawer-actions" aria-label="Estado de categoría">
        <button type="button" :disabled="disabled || stale" @click="emit('archive')">{{ category.archived ? 'Restaurar categoría' : 'Archivar categoría' }}</button>
        <p class="drawer-note">Archivar oculta la categoría de la planificación activa y conserva sus referencias históricas.</p>
        <p v-if="!category.can_delete" class="drawer-note">No se puede eliminar permanentemente: existen transacciones o historial presupuestario.</p>
        <label v-else class="delete-confirmation"><input type="checkbox" :checked="deleteConfirmed" :disabled="disabled" @change="emit('deleteConfirmation', ($event.target as HTMLInputElement).checked)"> Confirmar eliminación permanente</label>
        <button type="button" class="delete-button" :disabled="disabled || stale || !category.can_delete || !deleteConfirmed" @click="emit('remove')">Eliminar permanentemente</button>
      </section>
    </dialog>
  </Teleport>
</template>

<style scoped>
.category-drawer { position: fixed; inset: 0 0 0 auto; box-sizing: border-box; width: min(30rem, 100vw); max-width: 100vw; height: 100dvh; max-height: 100dvh; margin: 0; padding: var(--island-padding); color: var(--text); background: var(--surface); border: 0; border-left: 1px solid var(--border); box-shadow: var(--shadow-island); overflow-y: auto; }
.category-drawer::backdrop { background: rgb(0 0 0 / .4); }
.drawer-heading { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: start; gap: 1rem; margin-bottom: 1.5rem; }.drawer-heading h2 { margin: 0; font-size: 1.25rem; }
.drawer-form, .drawer-field { display: grid; gap: .6rem; }.drawer-form { gap: 1.2rem; }
.drawer-field input, .drawer-field select { box-sizing: border-box; width: 100%; min-height: 2.75rem; padding: .6rem; font: inherit; border: 1px solid var(--border); border-radius: var(--control-radius); background: var(--surface); color: var(--text); }
.drawer-actions { display: grid; gap: .75rem; border-top: 1px solid var(--border); margin-top: 1.5rem; padding-top: 1.5rem; }
.drawer-note { color: var(--muted); line-height: 1.6; }.drawer-error { padding: .75rem; border-radius: .4rem; background: var(--danger-bg); }
.delete-button { color: var(--danger); }.delete-confirmation { display: flex; align-items: center; gap: .5rem; }
button { min-height: 2.5rem; padding: .5rem .75rem; cursor: pointer; } button:disabled { opacity: .6; cursor: default; }
@media (max-width: 600px) { .category-drawer { width: 100vw; border-left: 0; } }
</style>
