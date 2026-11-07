<script setup lang="ts">
import { toRef, watch } from 'vue'
import type { ExpenseCategory } from '../api/contracts'
import { useBudgetCategories } from '../composables/use-budget-categories'

const props = defineProps<{
  categories: readonly ExpenseCategory[]
  disabled: boolean
  loadError: string
  refresh: () => Promise<void>
  restorePending: () => void
  errorMessage: (error: unknown, fallback: string) => string
}>()
const emit = defineEmits<{ busyChange: [busy: boolean] }>()
const workflow = useBudgetCategories({
  categories: toRef(props, 'categories'),
  refresh: props.refresh,
  restorePending: props.restorePending,
  errorMessage: props.errorMessage,
})
const { drafts: categoryDrafts, newCategory, error: categoryError, notice: categoryNotice } = workflow
watch(() => workflow.saving.value, (value) => emit('busyChange', value !== null), { immediate: true })

defineExpose({ confirmPending: workflow.confirmPending })
</script>

<template>
  <section class="panel category-manager" aria-labelledby="category-manager-title">
    <h2 id="category-manager-title">Administrar categorías</h2>
    <p>Añade categorías, cambia su nombre u orden y archiva las que todavía no tienen actividad ni asignaciones.</p>
    <p v-if="categoryError || loadError" class="notice notice-error" role="alert">{{ categoryError || loadError }}</p>
    <p v-if="categoryNotice" class="notice notice-success" role="status">{{ categoryNotice }}</p>
    <form class="category-new-form" @submit.prevent="workflow.saveNew">
      <label>Nombre<input v-model="newCategory.name" required maxlength="80" autocomplete="off"></label>
      <label>Orden<input v-model="newCategory.sort_order" required type="number" min="0" max="32767" step="1"></label>
      <button type="submit" :disabled="disabled">Añadir categoría</button>
    </form>
    <div class="category-list">
      <article v-for="category in categories" :key="category.id" class="category-item">
        <template v-if="categoryDrafts[category.id]">
          <label :for="`category-name-${category.id}`">Nombre</label><input :id="`category-name-${category.id}`" v-model="categoryDrafts[category.id].name" required maxlength="80">
          <label :for="`category-order-${category.id}`">Orden</label><input :id="`category-order-${category.id}`" v-model="categoryDrafts[category.id].sort_order" required type="number" min="0" max="32767" step="1">
          <span>{{ categoryDrafts[category.id].archived ? 'Archivada' : 'Activa' }}</span>
          <button type="button" :disabled="disabled" @click="workflow.save(category.id)">Guardar</button>
          <button type="button" :disabled="disabled" @click="workflow.setArchived(category.id, !category.archived)">{{ category.archived ? 'Restaurar' : 'Archivar' }}</button>
          <button type="button" class="text-button" :disabled="disabled" @click="workflow.cancel(category.id)">Cancelar</button>
        </template>
        <template v-else>
          <strong>{{ category.name }}</strong><span>Orden {{ category.sort_order }} · {{ category.archived ? 'archivada' : 'activa' }}</span>
          <button type="button" @click="workflow.edit(category)">Editar</button>
        </template>
      </article>
    </div>
  </section>
</template>

<style scoped>
.panel { background: var(--surface); border: 1px solid var(--border); min-width: 0; border-radius: var(--island-radius); padding: var(--island-padding); box-shadow: var(--shadow-island); }
.notice { padding: .75rem; background: var(--warning-bg); border-radius: .4rem; }.notice-error { background: var(--danger-bg); }.notice-success { background: var(--success-bg); }
.category-manager { display: grid; gap: .75rem; }.category-new-form, .category-item { display: flex; flex-wrap: wrap; align-items: end; gap: .65rem; }.category-new-form label, .category-item label { display: grid; gap: .25rem; }.category-new-form input, .category-item input { min-height: 2.5rem; padding: .4rem; }.category-new-form input { max-width: 9rem; padding: .45rem; }.category-list { display: grid; gap: .5rem; }.category-item { justify-content: space-between; border-top: 1px solid var(--border); padding-top: .6rem; }.category-item input { max-width: 12rem; }
button { min-height: 2.5rem; cursor: pointer; } button:disabled { cursor: wait; opacity: .65; }.text-button { border: 0; background: none; text-decoration: underline; }
</style>
