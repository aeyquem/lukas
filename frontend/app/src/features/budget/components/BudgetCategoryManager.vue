<script setup lang="ts">
import { computed, shallowRef, toRef, watch } from 'vue'
import type { BudgetGroup, ExpenseCategory } from '../api/contracts'
import { useBudgetCategories } from '../composables/use-budget-categories'
import BudgetCategoryDrawer from './budget-category-drawer.vue'
const props = withDefaults(defineProps<{
  categories: readonly ExpenseCategory[]; groups: readonly BudgetGroup[]; disabled: boolean; loadError: string
  refresh: () => Promise<void>; restorePending: () => void; errorMessage: (error: unknown, fallback: string) => string
  pendingRoutes?: readonly string[]
}>(), { pendingRoutes: () => [] })
const emit = defineEmits<{ busyChange: [busy: boolean] }>()
const workflow = useBudgetCategories({ categories: toRef(props, 'categories'), refresh: props.refresh, restorePending: props.restorePending, errorMessage: props.errorMessage })
const { drafts, newCategory, error, notice } = workflow
const open = shallowRef(false)
const selectedId = shallowRef<string | null>(null)
const deleteConfirmed = shallowRef(false)
const selectedCategory = computed(() => props.categories.find((category) => category.id === selectedId.value) ?? null)
const draft = computed(() => selectedId.value ? drafts.value[selectedId.value] ?? null : newCategory.value)
const stale = computed(() => Boolean(selectedCategory.value && draft.value && 'expected_revision' in draft.value && draft.value.expected_revision !== selectedCategory.value.revision))
const categoryList = computed(() => [...props.categories].sort((a, b) => Number(a.archived) - Number(b.archived) || a.sort_order - b.sort_order || a.id.localeCompare(b.id)))
const pending = computed(() => props.pendingRoutes.some((route) => route === '/api/categories' || route.startsWith('/api/categories/')))
const disabled = computed(() => props.disabled || pending.value)
watch(() => workflow.saving.value, (value) => emit('busyChange', value !== null), { immediate: true })
function openCategory(categoryId: string): void {
  const category = props.categories.find((item) => item.id === categoryId)
  if (!category) return
  workflow.edit(category)
  selectedId.value = categoryId; deleteConfirmed.value = false; open.value = true
}
function openNew(): void {
  selectedId.value = null
  if (!props.groups.some((group) => group.id === newCategory.value.group_id && !group.archived)) newCategory.value.group_id = props.groups.find((group) => !group.archived)?.id ?? ''
  error.value = ''; deleteConfirmed.value = false; open.value = true
}
function nameChanged(name: string): void { if (draft.value) draft.value.name = name }
function groupChanged(groupId: string): void { if (draft.value) draft.value.group_id = groupId }
async function save(): Promise<void> {
  if (disabled.value || stale.value) return
  if (await workflow.save(selectedId.value)) open.value = false
}
async function archive(): Promise<void> {
  if (!selectedCategory.value || disabled.value || stale.value) return
  if (await workflow.setArchived(selectedCategory.value.id, !selectedCategory.value.archived)) open.value = false
}
async function remove(): Promise<void> {
  if (!selectedCategory.value || !deleteConfirmed.value || disabled.value || stale.value) return
  if (await workflow.remove(selectedCategory.value.id)) open.value = false
}
async function confirmPending(route: string): Promise<void> { await workflow.confirmPending(route); open.value = false }
defineExpose({ openCategory, openNew, confirmPending })
</script>

<template>
  <section class="panel category-manager" aria-labelledby="category-manager-title">
    <header class="manager-heading"><h2 id="category-manager-title">Administrar categorías</h2><button type="button" :disabled="disabled" @click="openNew">Añadir categoría</button></header>
    <p>Administra nombres, grupos y categorías archivadas. El orden se cambia en el modo de organización del presupuesto.</p>
    <p v-if="loadError || (error && !open)" class="notice notice-error" role="alert">{{ loadError || error }}</p>
    <p v-if="notice" class="notice notice-success" role="status">{{ notice }}</p>
    <p v-if="pending" class="notice" role="status">Hay un cambio de categoría pendiente. Confirma su resultado antes de enviar otro.</p>
    <ul class="category-list"><li v-for="category in categoryList" :key="category.id" class="category-item"><div><strong>{{ category.name }}</strong><span class="category-state">{{ groups.find((group) => group.id === category.group_id)?.name ?? 'Grupo no disponible' }} · {{ category.archived ? 'Archivada' : 'Activa' }}</span></div><button type="button" :aria-label="`Editar ${category.name}`" @click="openCategory(category.id)">Editar</button></li></ul>
    <BudgetCategoryDrawer :open="open" :category="selectedCategory" :draft="draft" :groups="groups" :disabled="disabled" :error="error" :stale="stale" :delete-confirmed="deleteConfirmed" @close="open = false" @save="save" @archive="archive" @remove="remove" @review="selectedId && workflow.review(selectedId)" @name-change="nameChanged" @group-change="groupChanged" @delete-confirmation="deleteConfirmed = $event" />
  </section>
</template>

<style scoped>
.panel { background: var(--surface); border: 1px solid var(--border); min-width: 0; border-radius: var(--island-radius); padding: var(--island-padding); box-shadow: var(--shadow-island); }
.category-manager { display: grid; gap: .75rem; }.manager-heading { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 1rem; }.manager-heading h2 { margin: 0; }
.category-list { list-style: none; display: grid; gap: .5rem; padding: 0; margin: 0; }.category-item { display: flex; align-items: center; justify-content: space-between; gap: .75rem; border-top: 1px solid var(--border); padding-top: .75rem; }
.category-state { display: block; margin-top: .3rem; color: var(--muted); }.notice { padding: .75rem; background: var(--warning-bg); border-radius: .4rem; }.notice-error { background: var(--danger-bg); }.notice-success { background: var(--success-bg); }
button { min-height: 2.5rem; cursor: pointer; } button:disabled { opacity: .65; cursor: default; }
</style>
