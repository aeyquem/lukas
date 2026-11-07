<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useQueryCache } from '@pinia/colada'
import { ApiClientError } from '../shared/api/client'
import { formatMoney } from '../shared/formatting/money'
import { dismissPendingWrite, PendingWriteStore, type PendingFinancialWrite } from '../shared/api/pending-commands'
import type { BudgetClosePreview, BudgetValue } from '../features/budget/api/contracts'
import { categoryOverused, currentPeriodInSantiago, prepareCloseWrite, unresolvedDescriptions, usagePercent } from '../features/budget/budget-model'
import { fetchBudgetSnapshot, useBudgetClosePreviewQuery, useBudgetSnapshotQuery, useExpenseCategoriesQuery } from '../features/budget/api/queries'
import { useBudgetRetryMutation, useBudgetWriteMutation } from '../features/budget/api/mutations'
import BudgetCategoryManager from '../features/budget/components/BudgetCategoryManager.vue'
import BudgetClosePanel from '../features/budget/components/BudgetClosePanel.vue'
import { useBudgetAssignments } from '../features/budget/composables/use-budget-assignments'
const route = useRoute()
const router = useRouter()
const queryCache = useQueryCache()
const initialPeriod = typeof route.query.period === 'string' && /^\d{4}-(0[1-9]|1[0-2])$/.test(route.query.period) ? route.query.period : currentPeriodInSantiago()
const period = ref(initialPeriod)
const budgetQuery = useBudgetSnapshotQuery(period)
const categoriesQuery = useExpenseCategoriesQuery()
const closePreviewQuery = useBudgetClosePreviewQuery(period)
const budgetWrite = useBudgetWriteMutation()
const budgetRetry = useBudgetRetryMutation()
const snapshotLoadedPeriod = ref<string | null>(null)
const categoriesLoaded = ref(false)
const budget = computed(() => snapshotLoadedPeriod.value === period.value ? budgetQuery.data.value ?? null : null)
const categories = computed(() => categoriesLoaded.value ? categoriesQuery.data.value ?? [] : [])
const preview = ref<BudgetClosePreview | null>(null)
const dispositions = ref<Record<string, 'carry' | 'release'>>({})
const pending = ref<PendingFinancialWrite[]>([])
const dismissedPending = ref(new Set<string>())
const loading = ref(true)
const saving = ref<string | null>(null)
const pageError = ref('')
const staleMessage = ref('')
const success = ref('')
let pendingStore: PendingWriteStore | undefined
const sortedCategories = computed(() => [...(budget.value?.categories ?? [])].sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name)))
const visiblePending = computed(() => pending.value.filter((item) => !dismissedPending.value.has(item.idempotencyKey)))
const categoryLoadError = ref('')
const categoryBusy = ref(false)
const categoryManager = ref<InstanceType<typeof BudgetCategoryManager> | null>(null)
const assignmentWorkflow = useBudgetAssignments({
  period,
  budget,
  refresh,
  restorePending,
  errorMessage: message,
  onError: (value) => { pageError.value = value },
  onStale: (value) => { staleMessage.value = value },
  onSuccess: (value) => { success.value = value },
})
const { drafts, reasonDrafts, draftKey, assignDraft, assignReason } = assignmentWorkflow
const busy = computed(() => saving.value !== null || assignmentWorkflow.saving.value !== null || categoryBusy.value)
let latestLoad = 0
let latestPreview = 0

onMounted(() => { restorePending(); void loadInitialData() })
watch(() => route.query.period, async (queryPeriod) => {
  if (typeof queryPeriod !== 'string' || !/^\d{4}-(0[1-9]|1[0-2])$/.test(queryPeriod) || queryPeriod === period.value) return
  budgetQuery.suspend()
  period.value = queryPeriod
  latestPreview += 1
  preview.value = null
  snapshotLoadedPeriod.value = null
  pageError.value = ''
  loading.value = true
  const loadId = ++latestLoad
  try {
    await budgetQuery.refetchFresh()
    if (period.value === queryPeriod && loadId === latestLoad) {
      snapshotLoadedPeriod.value = queryPeriod
      staleMessage.value = ''
    }
  } catch (error) {
    if (period.value === queryPeriod && loadId === latestLoad) pageError.value = message(error, 'No fue posible cargar el presupuesto.')
  } finally {
    budgetQuery.resume()
    if (loadId === latestLoad) loading.value = false
  }
})

async function loadInitialData(): Promise<void> {
  loading.value = true
  pageError.value = ''
  const [snapshot, categoryList] = await Promise.allSettled([
    budgetQuery.activate(),
    categoriesQuery.activate(),
  ])
  if (snapshot.status === 'fulfilled') snapshotLoadedPeriod.value = period.value
  else pageError.value = message(snapshot.reason, 'No fue posible cargar el presupuesto.')
  if (categoryList.status === 'fulfilled') categoriesLoaded.value = true
  else categoryLoadError.value = message(categoryList.reason, 'No fue posible cargar las categorías.')
  loading.value = false
}

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
    await budgetQuery.refetchFresh()
    if (period.value !== requestedPeriod || loadId !== latestLoad) return
    snapshotLoadedPeriod.value = requestedPeriod
    // Drafts are intentionally retained across refetches, including stale conflicts.
    staleMessage.value = ''
  } catch (error) {
    if (period.value === requestedPeriod && loadId === latestLoad) {
      pageError.value = message(error, 'No fue posible cargar el presupuesto.')
    }
  }
  finally { if (loadId === latestLoad) loading.value = false }
}

function periodChanged(value: string): void {
  if (!/^\d{4}-(0[1-9]|1[0-2])$/.test(value)) return
  void router.replace({ name: 'budget', query: { period: value } })
}
function setDisposition(categoryId: string, disposition: 'carry' | 'release'): void {
  dispositions.value = { ...dispositions.value, [categoryId]: disposition }
}

async function refreshCategories(): Promise<void> {
  try {
    await categoriesQuery.refetchFresh()
    categoriesLoaded.value = true
    categoryLoadError.value = ''
  } catch (error) {
    categoryLoadError.value = message(error, 'No fue posible cargar las categorías.')
  }
}

async function loadClosePreview(): Promise<void> {
  const requestedPeriod = period.value
  const previewId = ++latestPreview
  pageError.value = ''; preview.value = null
  try {
    const result = await closePreviewQuery.refetch(true)
    if (period.value !== requestedPeriod || previewId !== latestPreview) return
    if (result.status !== 'success') throw result.error ?? new Error('No fue posible cargar la vista previa.')
    preview.value = result.data
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
  const body = prepareCloseWrite(current, dispositions.value)
  if (!body) return
  pageError.value = ''; success.value = ''; saving.value = 'close'
  try {
    await budgetWrite.mutateAsync({ method: 'POST', route: `/api/budgets/${closedPeriod}/close`, body })
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
    await budgetRetry.mutateAsync(command)
    restorePending()
    const commandPeriod = command.route.match(/\/budgets\/(\d{4}-\d{2})\//)?.[1]
    if (command.route === '/api/categories' || command.route.startsWith('/api/categories/')) {
      await categoryManager.value?.confirmPending(command.route)
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
  try {
    await fetchBudgetSnapshot(queryCache, nextPeriod)
    if (period.value === closedPeriod) {
      period.value = nextPeriod
      snapshotLoadedPeriod.value = nextPeriod
      budgetQuery.resume()
      await router.replace({ name: 'budget', query: { period: nextPeriod } })
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
    <section v-if="visiblePending.length" class="panel" aria-labelledby="pending-title"><h2 id="pending-title">Comandos pendientes</h2><p>El resultado de estos comandos todavía no se confirmó. Puedes reintentar el mismo envío.</p><ul><li v-for="command in visiblePending" :key="command.idempotencyKey"><span>{{ command.route.startsWith('/api/categories') ? 'Categoría' : command.route.includes('/close') ? 'Cierre' : 'Asignación' }} · {{ command.createdAt }}</span><button type="button" :disabled="busy" @click="retry(command)">Reintentar comando exacto</button><button type="button" :disabled="busy" @click="dismiss(command)">Ocultar aviso</button></li></ul></section>
    <BudgetCategoryManager ref="categoryManager" :categories="categories" :disabled="busy" :load-error="categoryLoadError" :refresh="refreshCategories" :restore-pending="restorePending" :error-message="message" @busy-change="categoryBusy = $event" />
    <p v-if="loading" role="status">Cargando presupuesto…</p>
    <template v-if="budget">
      <section class="summary-grid" aria-label="Resumen del período">
        <article class="panel summary-card"><h2>Financiamiento</h2><strong class="numeric">{{ valueText(budget.funding) }}</strong><ul v-if="explanation(budget.funding).length"><li v-for="detail in explanation(budget.funding)" :key="detail">{{ detail }}</li></ul></article>
        <article class="panel summary-card"><h2>Listo para asignar</h2><strong class="numeric">{{ valueText(budget.ready_to_assign) }}</strong><ul v-if="explanation(budget.ready_to_assign).length"><li v-for="detail in explanation(budget.ready_to_assign)" :key="detail">{{ detail }}</li></ul></article>
        <article class="panel summary-card"><h2>Actividad</h2><strong class="numeric">{{ valueText(budget.activity) }}</strong><ul v-if="explanation(budget.activity).length"><li v-for="detail in explanation(budget.activity)" :key="detail">{{ detail }}</li></ul></article>
        <article class="panel summary-card unavailable-summary"><h2>Ingresos del período</h2><strong>No disponible</strong><p>El resumen del presupuesto todavía no devuelve este total.</p></article>
        <article class="panel summary-card unavailable-summary"><h2>Saldos negativos sin arrastre</h2><strong>No disponible</strong><p>La política de arrastre por categoría aún no está definida.</p></article>
      </section>
      <section class="panel" aria-labelledby="categories-title"><h2 id="categories-title">Categorías</h2><p v-if="!budget.closed">Los saldos restantes conservan su signo; un valor negativo indica gasto por encima de lo asignado.</p><p v-else>Las asignaciones originales del cierre permanecen como referencia. Las correcciones guardan el objetivo vigente y su efecto neto por separado.</p>
        <div class="table-scroll"><table><thead><tr><th scope="col">Categoría</th><th scope="col">Asignado</th><th scope="col">Actividad</th><th scope="col">Restante</th><th scope="col">Nueva asignación</th></tr></thead>
          <tbody><tr v-for="category in sortedCategories" :key="category.id"><th scope="row">{{ category.name }}<div class="usage-indicator" :class="{ 'is-overused': categoryOverused(category) }" role="img" :aria-label="usagePercent(category) === null ? `Uso de ${category.name} incompleto` : categoryOverused(category) ? `${category.name} sobrepasó su asignación` : `${category.name}: ${usagePercent(category)} por ciento utilizado`"><span v-if="usagePercent(category) !== null" :style="{ width: `${usagePercent(category)}%` }"></span></div><small v-if="categoryOverused(category)" class="overused-label">Sobrepasado</small><small v-else-if="usagePercent(category) === null" class="detail">Uso incompleto</small><small v-else class="detail">{{ usagePercent(category) }}% utilizado</small></th><td class="numeric">{{ money(category.assigned) }}<small v-if="budget.closed" class="detail">Objetivo vigente: {{ money(category.assignment_intent ?? category.assigned) }}</small></td><td class="numeric">{{ valueText(category.activity) }}<small v-for="detail in explanation(category.activity)" :key="detail" class="detail">{{ detail }}</small></td><td class="numeric">{{ valueText(category.remaining) }}<small v-for="detail in explanation(category.remaining)" :key="detail" class="detail">{{ detail }}</small></td><td><form class="assignment-form" @submit.prevent="assignmentWorkflow.save(category.id)"><label class="sr-only" :for="`assign-${category.id}`">{{ budget.closed ? 'Nuevo objetivo corregido' : 'Nueva asignación' }} para {{ category.name }} en CLP</label><input :id="`assign-${category.id}`" :value="drafts[draftKey(category.id)] ?? (budget.closed ? category.assignment_intent ?? category.assigned : category.assigned)" inputmode="decimal" autocomplete="off" @input="assignDraft(category.id, ($event.target as HTMLInputElement).value)"><label v-if="budget.closed" class="reason-field">Motivo de corrección<textarea :value="reasonDrafts[draftKey(category.id)] ?? ''" required @input="assignReason(category.id, ($event.target as HTMLTextAreaElement).value)"></textarea></label><button type="submit" :disabled="busy">{{ budget.closed ? 'Guardar corrección' : 'Guardar' }}</button></form></td></tr></tbody>
        </table></div>
        <div class="uncategorized"><h3>Actividad sin categoría</h3><strong class="numeric">{{ valueText(budget.uncategorized_activity) }}</strong><ul v-if="explanation(budget.uncategorized_activity).length"><li v-for="detail in explanation(budget.uncategorized_activity)" :key="detail">{{ detail }}</li></ul></div>
      </section>
      <BudgetClosePanel v-if="!budget.closed" :preview="preview" :dispositions="dispositions" :disabled="busy" @load-preview="loadClosePreview" @close="closePeriod" @update-disposition="setDisposition" />
    </template>
  </main>
</template>

<style scoped>
.budget-page { display: grid; gap: var(--island-gap); padding-block: 8px 24px; min-width: 0; }
.page-heading { display: flex; justify-content: space-between; align-items: end; gap: 1rem; }
.page-heading h1 { margin: 0; }.page-heading p { margin: .35rem 0 0; }
.eyebrow { text-transform: uppercase; letter-spacing: .08em; font-size: .75rem; }
.summary-grid { display: grid; grid-template-columns: repeat(auto-fit,minmax(12rem,1fr)); gap: var(--island-gap); }
.panel { background: var(--surface); border: 1px solid var(--border); min-width: 0; border-radius: var(--island-radius); padding: var(--island-padding); box-shadow: var(--shadow-island); }
.summary-card h2 { font-size: 1rem; margin: 0 0 .7rem; }.summary-card strong { font-size: 1.25rem; }
.table-scroll { position: relative; min-width: 0; overflow-x: auto; } table { width: 100%; border-collapse: collapse; } th,td { padding: .65rem; border-bottom: 1px solid var(--border); text-align: left; vertical-align: top; } thead th { white-space: nowrap; }
.assignment-form { display: flex; gap: .4rem; } input:not([type=radio]) { max-width: 9rem; padding: .45rem; }
button { min-height: 2.5rem; cursor: pointer; } button:disabled { cursor: wait; opacity: .65; }
.detail, .detail-list { display: block; color: var(--muted); font-size: .88rem; margin-top: .25rem; }.detail-list { padding-left: 1.2rem; }
.uncategorized { border-top: 1px solid var(--border); margin-top: 1rem; padding-top: .5rem; }.uncategorized h3 { margin-bottom: .3rem; }
.notice { padding: .75rem; background: var(--warning-bg); border-radius: .4rem; }.notice-error { background: var(--danger-bg); }.notice-success { background: var(--success-bg); }
.reason-field { display: grid; gap: .25rem; }.reason-field textarea { min-height: 2.5rem; padding: .4rem; }
.text-button { border: 0; background: none; text-decoration: underline; }.sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border: 0; }
@media (max-width: 650px) { .page-heading { align-items: start; flex-direction: column; } }
.unavailable-summary strong { display: block; font-size: 1rem; color: var(--muted); }
.unavailable-summary p { margin: .45rem 0 0; color: var(--muted); font-size: .82rem; line-height: 1.45; }
.usage-indicator { width: min(150px, 100%); height: 6px; margin-top: .55rem; overflow: hidden; border-radius: 999px; background: var(--border); }
.usage-indicator span { display: block; height: 100%; border-radius: inherit; background: var(--accent); }
.usage-indicator.is-overused span { background: var(--danger); }
.overused-label { display: block; margin-top: .3rem; color: var(--danger); font-size: .75rem; font-weight: 650; }
</style>
