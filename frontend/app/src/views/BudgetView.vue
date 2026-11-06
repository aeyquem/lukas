<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ApiClientError, apiRequest } from '../shared/api/client'
import { formatMoney } from '../shared/formatting/money'
import { sendFinancialWrite, retryPendingWrite, dismissPendingWrite, PendingWriteStore, type PendingFinancialWrite } from '../shared/api/pending-commands'
import type { BudgetSnapshot, BudgetValue } from '../components/budget/budget-model'
import { budgetRoute, currentPeriodInSantiago, normalizeCategoryDraft, unresolvedDescriptions } from '../components/budget/budget-model'

interface ClosePreview extends BudgetSnapshot {
  close: {
    eligible: boolean
    reason: string | null
    expected_period_revision: string
    categories: Array<{ category_id: string; proposed_disposition: 'carry' }>
  }
}
interface Receipt { resource_id: string; revision: number }
interface ExpenseCategory {
  id: string
  slug: string
  name: string
  sort_order: number
  archived: boolean
}
interface CategoryDraft { name: string; sort_order: string; archived: boolean }
const period = ref(currentPeriodInSantiago())
const budget = ref<BudgetSnapshot | null>(null)
const drafts = ref<Record<string, string>>({})
const reasonDrafts = ref<Record<string, string>>({})
const preview = ref<ClosePreview | null>(null)
const dispositions = ref<Record<string, 'carry' | 'release'>>({})
const pending = ref<PendingFinancialWrite[]>([])
const dismissedPending = ref(new Set<string>())
const categories = ref<ExpenseCategory[]>([])
const categoryDrafts = ref<Record<string, CategoryDraft>>({})
const newCategory = ref({ name: '', sort_order: '100' })
const categoryError = ref('')
const categoryNotice = ref('')
const loading = ref(false)
const saving = ref<string | null>(null)
const pageError = ref('')
const staleMessage = ref('')
const success = ref('')
let pendingStore: PendingWriteStore | undefined
const sortedCategories = computed(() => [...(budget.value?.categories ?? [])].sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name)))
const visiblePending = computed(() => pending.value.filter((item) => !dismissedPending.value.has(item.idempotencyKey)))
let latestLoad = 0
let latestPreview = 0

onMounted(() => { restorePending(); void refresh(); void refreshCategories() })

function getPendingStore(): PendingWriteStore {
  pendingStore ??= new PendingWriteStore()
  return pendingStore
}

async function refresh(requestedPeriod = period.value): Promise<void> {
  if (requestedPeriod !== period.value) return
  const loadId = ++latestLoad
  loading.value = true
  pageError.value = ''
  try {
    const snapshot = await apiRequest<BudgetSnapshot>(budgetRoute(requestedPeriod))
    if (period.value !== requestedPeriod || loadId !== latestLoad) return
    budget.value = snapshot
    // Drafts are intentionally retained across refetches, including stale conflicts.
    staleMessage.value = ''
  } catch (error) {
    if (period.value === requestedPeriod && loadId === latestLoad) {
      pageError.value = message(error, 'No fue posible cargar el presupuesto.')
    }
  }
  finally { if (loadId === latestLoad) loading.value = false }
}

function draftKey(categoryId: string): string { return `${period.value}:${categoryId}` }
function assignDraft(categoryId: string, value: string): void { drafts.value = { ...drafts.value, [draftKey(categoryId)]: value } }
function assignReason(categoryId: string, value: string): void { reasonDrafts.value = { ...reasonDrafts.value, [draftKey(categoryId)]: value } }
function periodChanged(value: string): void {
  if (!/^\d{4}-(0[1-9]|1[0-2])$/.test(value)) return
  period.value = value
  latestPreview += 1
  preview.value = null
  budget.value = null
  void refresh()
}

async function refreshCategories(): Promise<void> {
  try {
    categories.value = await apiRequest<ExpenseCategory[]>('categories')
  } catch (error) {
    categoryError.value = message(error, 'No fue posible cargar las categorías.')
  }
}

function editCategory(category: ExpenseCategory): void {
  categoryDrafts.value = {
    ...categoryDrafts.value,
    [category.id]: { name: category.name, sort_order: String(category.sort_order), archived: category.archived },
  }
}
function cancelCategoryEdit(categoryId: string): void {
  const next = { ...categoryDrafts.value }; delete next[categoryId]; categoryDrafts.value = next
}

async function saveNewCategory(): Promise<void> {
  await saveCategory(null)
}

async function setCategoryArchived(categoryId: string, archived: boolean): Promise<void> {
  const category = categories.value.find((item) => item.id === categoryId)
  if (!category) return
  if (!categoryDrafts.value[categoryId]) editCategory(category)
  const currentDraft = categoryDrafts.value[categoryId]
  if (!currentDraft) return
  categoryDrafts.value = { ...categoryDrafts.value, [categoryId]: { ...currentDraft, archived } }
  await saveCategory(categoryId)
}

async function saveCategory(categoryId: string | null, archive?: boolean): Promise<void> {
  const draft = categoryId ? categoryDrafts.value[categoryId] : newCategory.value
  const normalized = normalizeCategoryDraft(draft?.name ?? '', draft?.sort_order ?? '')
  if (!normalized) {
    categoryError.value = 'Escribe un nombre y un orden entero entre 0 y 32767.'
    return
  }
  categoryError.value = ''; categoryNotice.value = ''; saving.value = categoryId ?? 'category-create'
  const body = JSON.stringify({
    ...normalized,
    ...(categoryId ? { archived: archive ?? categoryDrafts.value[categoryId]?.archived ?? false } : {}),
  })
  try {
    await sendFinancialWrite<Receipt>({
      method: categoryId ? 'PUT' : 'POST',
      route: categoryId ? `/api/categories/${categoryId}` : '/api/categories',
      body,
    })
    if (categoryId) {
      const next = { ...categoryDrafts.value }; delete next[categoryId]; categoryDrafts.value = next
    } else {
      newCategory.value = { name: '', sort_order: String(Math.max(0, ...categories.value.map((item) => item.sort_order)) + 10) }
    }
    categoryNotice.value = categoryId ? 'Categoría actualizada.' : 'Categoría añadida.'
    restorePending(); await refreshCategories()
  } catch (error) {
    categoryError.value = error instanceof ApiClientError && error.code === 'category_in_use'
      ? 'No se puede archivar: la categoría ya tiene actividad o asignaciones.'
      : message(error, 'No fue posible guardar la categoría.')
    restorePending()
  } finally { saving.value = null }
}

async function saveAssignment(categoryId: string): Promise<void> {
  if (!budget.value) return
  const requestedPeriod = period.value
  const key = draftKey(categoryId)
  const category = budget.value.categories.find((item) => item.id === categoryId)
  const amount = drafts.value[key]
  if (!category || amount === undefined || !/^\d+(?:\.\d+)?$/.test(amount)) return
  const reason = reasonDrafts.value[key] ?? ''
  if (budget.value.closed && !reason.trim()) {
    pageError.value = 'Indica el motivo de la corrección para un período cerrado.'
    return
  }
  saving.value = categoryId; pageError.value = ''; staleMessage.value = ''; success.value = ''
  const body = JSON.stringify({ amount, expected_period_revision: budget.value.revision, expected_revision: category.assignment_revision, ...(budget.value.closed ? { reason: reason.trim() } : {}) })
  try {
    await sendFinancialWrite<Receipt>({ method: 'PUT', route: `/api/budgets/${requestedPeriod}/assignments/${categoryId}`, body })
    const nextDrafts = { ...drafts.value }; delete nextDrafts[key]; drafts.value = nextDrafts
    const nextReasons = { ...reasonDrafts.value }; delete nextReasons[key]; reasonDrafts.value = nextReasons
    success.value = `Asignación guardada para ${category.name}.`
    restorePending(); await refresh(requestedPeriod)
  } catch (error) {
    if (error instanceof ApiClientError && error.kind === 'conflict' && period.value === requestedPeriod) {
      const conflictMessage = `El presupuesto cambió. El borrador de ${category.name} se conserva; actualiza los datos y vuelve a guardar.`
      await refresh(requestedPeriod)
      if (period.value === requestedPeriod) staleMessage.value = conflictMessage
    }
    restorePending()
    if (period.value === requestedPeriod) pageError.value = message(error, 'No fue posible guardar la asignación.')
  } finally { saving.value = null }
}

async function loadClosePreview(): Promise<void> {
  const requestedPeriod = period.value
  const previewId = ++latestPreview
  pageError.value = ''; preview.value = null
  try {
    const result = await apiRequest<ClosePreview>(budgetRoute(requestedPeriod, '/close-preview'))
    if (period.value !== requestedPeriod || previewId !== latestPreview) return
    preview.value = result
    dispositions.value = Object.fromEntries(preview.value.close.categories.map((item) => [item.category_id, item.proposed_disposition]))
  } catch (error) {
    if (period.value === requestedPeriod && previewId === latestPreview) {
      pageError.value = message(error, 'No fue posible cargar la vista previa del cierre.')
    }
  }
}

async function closePeriod(): Promise<void> {
  const current = preview.value
  if (!current?.close.eligible) return
  const closedPeriod = current.period
  pageError.value = ''; success.value = ''; saving.value = 'close'
  const body = JSON.stringify({ expected_period_revision: current.close.expected_period_revision, categories: current.close.categories.map((item) => ({ category_id: item.category_id, disposition: dispositions.value[item.category_id] ?? 'carry' })) })
  try {
    await sendFinancialWrite<Receipt>({ method: 'POST', route: `/api/budgets/${closedPeriod}/close`, body })
    success.value = 'El período se cerró. Se actualizará el resumen autoritativo.'
    preview.value = null; restorePending()
    await refreshAfterClose(closedPeriod)
  } catch (error) {
    if (error instanceof ApiClientError && error.kind === 'conflict') {
      staleMessage.value = 'La vista previa quedó desactualizada. Se conserva la selección; vuelve a cargar la vista previa antes de confirmar.'
    }
    restorePending(); pageError.value = message(error, 'No fue posible cerrar el período.')
  } finally { saving.value = null }
}

async function retry(command: PendingFinancialWrite): Promise<void> {
  saving.value = command.idempotencyKey; pageError.value = ''
  try {
    await retryPendingWrite<Receipt>(command.idempotencyKey)
    restorePending()
    const commandPeriod = command.route.match(/\/budgets\/(\d{4}-\d{2})\//)?.[1]
    if (command.route === '/api/categories' || command.route.startsWith('/api/categories/')) {
      await refreshCategories()
      const categoryId = command.route.match(/^\/api\/categories\/([^/]+)$/)?.[1]
      if (categoryId) cancelCategoryEdit(categoryId)
      else newCategory.value = { name: '', sort_order: String(Math.max(0, ...categories.value.map((item) => item.sort_order)) + 10) }
      categoryNotice.value = 'El cambio de categoría pendiente se confirmó.'
    } else if (command.route.includes('/close')) {
      if (commandPeriod) await refreshAfterClose(commandPeriod)
      success.value = 'El cierre guardado se confirmó.'
    } else {
      if (commandPeriod && commandPeriod === period.value) await refresh(commandPeriod)
      success.value = 'La asignación guardada se confirmó.'
    }
  } catch (error) { pageError.value = message(error, 'No se pudo confirmar el comando pendiente.'); restorePending() }
  finally { saving.value = null }
}

async function refreshAfterClose(closedPeriod: string): Promise<void> {
  const nextPeriod = nextMonth(closedPeriod)
  if (period.value === closedPeriod && budget.value) {
    budget.value = { ...budget.value, closed: true }
  }
  try {
    const [closed, next] = await Promise.all([
      apiRequest<BudgetSnapshot>(budgetRoute(closedPeriod)),
      apiRequest<BudgetSnapshot>(budgetRoute(nextPeriod)),
    ])
    void closed
    if (period.value === closedPeriod) {
      period.value = nextPeriod
      budget.value = next
    }
  } catch (error) {
    pageError.value = `El período se cerró, pero no se pudo actualizar el resumen: ${message(error, 'error de lectura')}`
  }
}

function dismiss(command: PendingFinancialWrite): void {
  try {
    dismissPendingWrite(command.idempotencyKey, getPendingStore())
    dismissedPending.value = new Set([...dismissedPending.value, command.idempotencyKey])
  } catch (error) { pageError.value = message(error, 'No fue posible ocultar el aviso.') }
}

function restorePending(): void {
  try { pending.value = getPendingStore().list().filter((command) => command.route.startsWith('/api/budgets/') || command.route.startsWith('/api/categories')) }
  catch (error) { pageError.value = message(error, 'No fue posible leer los comandos guardados.') }
}
function money(value: string): string { return formatMoney(value, 'CLP') }
function valueText(value: BudgetValue): string {
  if (value.state === 'complete') return money(value.amount ?? '0')
  if (value.state === 'unavailable') return 'No disponible'
  return `${money(value.known_subtotal ?? '0')} · subtotal conocido`
}
function explanation(value: BudgetValue): string[] {
  if (value.state === 'unavailable') return value.reasons ?? []
  if (value.state === 'incomplete') return unresolvedDescriptions(value)
  return []
}
function message(error: unknown, fallback: string): string { return error instanceof Error ? error.message : fallback }
function nextMonth(value: string): string { const [year, month] = value.split('-').map(Number); return month === 12 ? `${year + 1}-01` : `${year}-${String(month + 1).padStart(2, '0')}` }
</script>

<template>
  <main class="page budget-page">
    <header class="page-heading"><div><p class="eyebrow">Planificación</p><h1>Presupuesto</h1><p>Período {{ period }} · valores presupuestarios en CLP<span v-if="budget?.closed"> · cerrado</span>.</p><label for="budget-period">Período</label> <input id="budget-period" type="month" :value="period" @change="periodChanged(($event.target as HTMLInputElement).value)"></div><button type="button" class="button-secondary" @click="refresh()">Actualizar</button></header>
    <p v-if="pageError" class="notice notice-error" role="alert">{{ pageError }}</p>
    <p v-if="staleMessage" class="notice" role="status">{{ staleMessage }} <button type="button" class="text-button" @click="refresh()">Actualizar presupuesto</button></p>
    <p v-if="success" class="notice notice-success" role="status">{{ success }}</p>
    <section v-if="visiblePending.length" class="panel" aria-labelledby="pending-title"><h2 id="pending-title">Comandos pendientes</h2><p>El resultado de estos comandos todavía no se confirmó. Puedes reintentar el mismo envío.</p><ul><li v-for="command in visiblePending" :key="command.idempotencyKey"><span>{{ command.route.startsWith('/api/categories') ? 'Categoría' : command.route.includes('/close') ? 'Cierre' : 'Asignación' }} · {{ command.createdAt }}</span><button type="button" :disabled="saving !== null" @click="retry(command)">Reintentar comando exacto</button><button type="button" :disabled="saving !== null" @click="dismiss(command)">Ocultar aviso</button></li></ul></section>
    <section class="panel category-manager" aria-labelledby="category-manager-title">
      <h2 id="category-manager-title">Administrar categorías</h2>
      <p>Añade categorías, cambia su nombre u orden y archiva las que todavía no tienen actividad ni asignaciones.</p>
      <p v-if="categoryError" class="notice notice-error" role="alert">{{ categoryError }}</p>
      <p v-if="categoryNotice" class="notice notice-success" role="status">{{ categoryNotice }}</p>
      <form class="category-new-form" @submit.prevent="saveNewCategory">
        <label>Nombre<input v-model="newCategory.name" required maxlength="80" autocomplete="off"></label>
        <label>Orden<input v-model="newCategory.sort_order" required type="number" min="0" max="32767" step="1"></label>
        <button type="submit" :disabled="saving !== null">Añadir categoría</button>
      </form>
      <div class="category-list">
        <article v-for="category in categories" :key="category.id" class="category-item">
          <template v-if="categoryDrafts[category.id]">
            <label :for="`category-name-${category.id}`">Nombre</label><input :id="`category-name-${category.id}`" v-model="categoryDrafts[category.id].name" required maxlength="80">
            <label :for="`category-order-${category.id}`">Orden</label><input :id="`category-order-${category.id}`" v-model="categoryDrafts[category.id].sort_order" required type="number" min="0" max="32767" step="1">
            <span>{{ categoryDrafts[category.id].archived ? 'Archivada' : 'Activa' }}</span>
            <button type="button" :disabled="saving !== null" @click="saveCategory(category.id)">Guardar</button>
            <button type="button" :disabled="saving !== null" @click="setCategoryArchived(category.id, !category.archived)">{{ category.archived ? 'Restaurar' : 'Archivar' }}</button>
            <button type="button" class="text-button" :disabled="saving !== null" @click="cancelCategoryEdit(category.id)">Cancelar</button>
          </template>
          <template v-else>
            <strong>{{ category.name }}</strong><span>Orden {{ category.sort_order }} · {{ category.archived ? 'archivada' : 'activa' }}</span>
            <button type="button" @click="editCategory(category)">Editar</button>
          </template>
        </article>
      </div>
    </section>
    <p v-if="loading" role="status">Cargando presupuesto…</p>
    <template v-if="budget">
      <section class="summary-grid" aria-label="Resumen del período">
        <article class="panel summary-card"><h2>Financiamiento</h2><strong>{{ valueText(budget.funding) }}</strong><ul v-if="explanation(budget.funding).length"><li v-for="detail in explanation(budget.funding)" :key="detail">{{ detail }}</li></ul></article>
        <article class="panel summary-card"><h2>Listo para asignar</h2><strong>{{ valueText(budget.ready_to_assign) }}</strong><ul v-if="explanation(budget.ready_to_assign).length"><li v-for="detail in explanation(budget.ready_to_assign)" :key="detail">{{ detail }}</li></ul></article>
        <article class="panel summary-card"><h2>Actividad</h2><strong>{{ valueText(budget.activity) }}</strong><ul v-if="explanation(budget.activity).length"><li v-for="detail in explanation(budget.activity)" :key="detail">{{ detail }}</li></ul></article>
      </section>
      <section class="panel" aria-labelledby="categories-title"><h2 id="categories-title">Categorías</h2><p v-if="!budget.closed">Los saldos restantes conservan su signo; un valor negativo indica gasto por encima de lo asignado.</p><p v-else>Las asignaciones originales del cierre permanecen como referencia. Las correcciones guardan el objetivo vigente y su efecto neto por separado.</p>
        <div class="table-scroll"><table><thead><tr><th scope="col">Categoría</th><th scope="col">Asignado</th><th scope="col">Actividad</th><th scope="col">Restante</th><th scope="col">Nueva asignación</th></tr></thead>
          <tbody><tr v-for="category in sortedCategories" :key="category.id"><th scope="row">{{ category.name }}</th><td>{{ money(category.assigned) }}<small v-if="budget.closed" class="detail">Objetivo vigente: {{ money(category.assignment_intent ?? category.assigned) }}</small></td><td>{{ valueText(category.activity) }}<small v-for="detail in explanation(category.activity)" :key="detail" class="detail">{{ detail }}</small></td><td>{{ valueText(category.remaining) }}<small v-for="detail in explanation(category.remaining)" :key="detail" class="detail">{{ detail }}</small></td><td><form class="assignment-form" @submit.prevent="saveAssignment(category.id)"><label class="sr-only" :for="`assign-${category.id}`">{{ budget.closed ? 'Nuevo objetivo corregido' : 'Nueva asignación' }} para {{ category.name }} en CLP</label><input :id="`assign-${category.id}`" :value="drafts[draftKey(category.id)] ?? (budget.closed ? category.assignment_intent ?? category.assigned : category.assigned)" inputmode="decimal" autocomplete="off" @input="assignDraft(category.id, ($event.target as HTMLInputElement).value)"><label v-if="budget.closed" class="reason-field">Motivo de corrección<textarea :value="reasonDrafts[draftKey(category.id)] ?? ''" required @input="assignReason(category.id, ($event.target as HTMLTextAreaElement).value)"></textarea></label><button type="submit" :disabled="saving !== null">{{ budget.closed ? 'Guardar corrección' : 'Guardar' }}</button></form></td></tr></tbody>
        </table></div>
        <div class="uncategorized"><h3>Actividad sin categoría</h3><strong>{{ valueText(budget.uncategorized_activity) }}</strong><ul v-if="explanation(budget.uncategorized_activity).length"><li v-for="detail in explanation(budget.uncategorized_activity)" :key="detail">{{ detail }}</li></ul></div>
      </section>
      <section v-if="!budget.closed" class="panel close-panel" aria-labelledby="close-title"><h2 id="close-title">Cerrar período</h2><p>Antes de cerrar, revisa cada saldo restante y elige si llevarlo al siguiente período o liberarlo.</p><button type="button" class="button-secondary" @click="loadClosePreview">Ver vista previa del cierre</button>
        <div v-if="preview" class="close-preview"><p v-if="!preview.close.eligible" class="notice" role="status">{{ preview.close.reason || 'El período todavía no se puede cerrar.' }}</p><fieldset v-for="item in preview.close.categories" :key="item.category_id"><legend>{{ preview.categories.find((category) => category.id === item.category_id)?.name ?? 'Categoría' }}</legend><label><input v-model="dispositions[item.category_id]" type="radio" :name="`disposition-${item.category_id}`" value="carry"> Llevar saldo</label><label><input v-model="dispositions[item.category_id]" type="radio" :name="`disposition-${item.category_id}`" value="release"> Liberar saldo</label></fieldset><button type="button" :disabled="!preview.close.eligible || saving !== null" @click="closePeriod">Confirmar cierre</button></div>
      </section>
    </template>
  </main>
</template>

<style scoped>
.budget-page { display: grid; gap: 1.25rem; }
.page-heading { display: flex; justify-content: space-between; align-items: end; gap: 1rem; }
.page-heading h1 { margin: 0; }.page-heading p { margin: .35rem 0 0; }
.eyebrow { text-transform: uppercase; letter-spacing: .08em; font-size: .75rem; }
.summary-grid { display: grid; grid-template-columns: repeat(auto-fit,minmax(12rem,1fr)); gap: 1rem; }
.panel { background: var(--surface, #fff); border: 1px solid var(--border, #d8d8d8); border-radius: .75rem; padding: 1rem; }
.summary-card h2 { font-size: 1rem; margin: 0 0 .7rem; }.summary-card strong { font-size: 1.25rem; }
.table-scroll { overflow-x: auto; } table { width: 100%; border-collapse: collapse; } th,td { padding: .65rem; border-bottom: 1px solid var(--border, #ddd); text-align: left; vertical-align: top; } thead th { white-space: nowrap; }
.assignment-form { display: flex; gap: .4rem; } input:not([type=radio]) { max-width: 9rem; padding: .45rem; }
button { min-height: 2.5rem; cursor: pointer; } button:disabled { cursor: wait; opacity: .65; }
.detail, .detail-list { display: block; color: var(--muted, #5b6470); font-size: .88rem; margin-top: .25rem; }.detail-list { padding-left: 1.2rem; }
.uncategorized { border-top: 1px solid var(--border, #ddd); margin-top: 1rem; padding-top: .5rem; }.uncategorized h3 { margin-bottom: .3rem; }
.notice { padding: .75rem; background: #fff5d8; border-radius: .4rem; }.notice-error { background: #fde8e7; }.notice-success { background: #e5f5e9; }
.close-preview { margin-top: 1rem; }.close-preview fieldset { display: flex; gap: 1rem; margin: .75rem 0; }.close-preview legend { font-weight: 600; }
.category-manager { display: grid; gap: .75rem; }.category-new-form, .category-item { display: flex; flex-wrap: wrap; align-items: end; gap: .65rem; }.category-new-form label, .category-item label { display: grid; gap: .25rem; }.category-new-form input, .category-item input, .reason-field textarea { min-height: 2.5rem; padding: .4rem; }.category-list { display: grid; gap: .5rem; }.category-item { justify-content: space-between; border-top: 1px solid var(--border, #ddd); padding-top: .6rem; }.category-item input { max-width: 12rem; }.reason-field { display: grid; gap: .25rem; }
.text-button { border: 0; background: none; text-decoration: underline; }.sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border: 0; }
@media (max-width: 650px) { .page-heading { align-items: start; flex-direction: column; }.close-preview fieldset { flex-direction: column; } }
</style>
