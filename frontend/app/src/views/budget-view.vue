<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useQueryCache } from '@pinia/colada'
import { ApiClientError } from '../shared/api/client'
import { dismissPendingWrite, PendingWriteStore, type PendingFinancialWrite } from '../shared/api/pending-commands'
import type { BudgetClosePreview } from '../features/budget/api/contracts'
import { currentPeriodInSantiago, prepareCloseWrite } from '../features/budget/budget-model'
import { fetchBudgetSnapshot, useBudgetClosePreviewQuery, useBudgetSnapshotQuery, useExpenseCategoriesQuery, useBudgetGroupsQuery } from '../features/budget/api/queries'
import { useBudgetRetryMutation, useBudgetWriteMutation } from '../features/budget/api/mutations'
import BudgetCategoryManager from '../features/budget/components/BudgetCategoryManager.vue'
import BudgetClosePanel from '../features/budget/components/BudgetClosePanel.vue'
import BudgetAvailableSummary from '../features/budget/components/budget-available-summary.vue'
import BudgetPlanningTable from '../features/budget/components/budget-planning-table.vue'
import BudgetGroupManager from '../features/budget/components/budget-group-manager.vue'
import { useBudgetAssignments } from '../features/budget/composables/use-budget-assignments'
const route = useRoute()
const router = useRouter()
const queryCache = useQueryCache()
const initialPeriod = typeof route.query.period === 'string' && /^\d{4}-(0[1-9]|1[0-2])$/.test(route.query.period) ? route.query.period : currentPeriodInSantiago()
const period = ref(initialPeriod)
const budgetQuery = useBudgetSnapshotQuery(period)
const categoriesQuery = useExpenseCategoriesQuery()
const groupsQuery = useBudgetGroupsQuery()
const tab = ref<'planning' | 'categories'>('planning')
const groups = computed(() => groupsQuery.data.value ?? budget.value?.groups ?? [])
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
const visiblePending = computed(() => pending.value.filter((item) => !dismissedPending.value.has(item.idempotencyKey)))
const categoryLoadError = ref('')
const categoryBusy = ref(false)
const organizationBusy = ref(false)
const groupBusy = ref(false)
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
const busy = computed(() => saving.value !== null || assignmentWorkflow.saving.value !== null || categoryBusy.value || organizationBusy.value || groupBusy.value)
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
  const [snapshot, categoryList, groupList] = await Promise.allSettled([
    budgetQuery.activate(),
    categoriesQuery.activate(),
    groupsQuery.activate(),
  ])
  if (snapshot.status === 'fulfilled') snapshotLoadedPeriod.value = period.value
  else pageError.value = message(snapshot.reason, 'No fue posible cargar el presupuesto.')
  if (categoryList.status === 'fulfilled') categoriesLoaded.value = true
  else categoryLoadError.value = message(categoryList.reason, 'No fue posible cargar las categorías.')
  if (groupList.status === 'rejected') categoryLoadError.value = message(groupList.reason, 'No fue posible cargar los grupos.')
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
    } else if (command.route.startsWith('/api/budget-groups')) {
      await refreshOrganization()
      success.value = 'El cambio de grupo pendiente se confirmó.'
    } else if (command.route.includes('/close')) {
      if (commandPeriod) await refreshAfterClose(commandPeriod)
      success.value = 'El cierre guardado se confirmó.'
    } else {
      assignmentWorkflow.confirmPending(command)
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
  try { pending.value = getPendingStore().list().filter((command) => command.route.startsWith('/api/budgets/') || command.route.startsWith('/api/categories') || command.route.startsWith('/api/budget-groups')) }
  catch (error) { pageError.value = message(error, 'No fue posible leer los comandos guardados.') }
}
async function refreshOrganization(): Promise<void> {
  const requestedPeriod = period.value
  loading.value = true
  try {
    await Promise.all([categoriesQuery.refetchFresh(), groupsQuery.refetchFresh()])
    categoriesLoaded.value = true
    categoryLoadError.value = ''
    await refresh(requestedPeriod)
    if (requestedPeriod === period.value && pageError.value) throw new Error(pageError.value)
  } finally {
    if (requestedPeriod === period.value) loading.value = false
  }
}

function message(error: unknown, fallback: string): string { return error instanceof Error ? error.message : fallback }
function nextMonth(value: string): string { const [year, month] = value.split('-').map(Number); return month === 12 ? `${year + 1}-01` : `${year}-${String(month + 1).padStart(2, '0')}` }

</script>

<template>
  <main class="page budget-page">
    <header class="page-heading"><div class="period-heading"><h1>Presupuesto</h1><label for="budget-period">Período</label><input id="budget-period" type="month" :value="period" @change="periodChanged(($event.target as HTMLInputElement).value)"></div><button type="button" class="button-secondary" :disabled="loading || busy" @click="refreshOrganization().catch((error) => categoryLoadError = message(error, 'No fue posible actualizar la organización.'))">Actualizar</button></header>
    <p v-if="pageError" class="notice notice-error" role="alert">{{ pageError }}</p>
    <p v-if="staleMessage" class="notice" role="status">{{ staleMessage }} <button type="button" class="text-button" @click="refresh()">Actualizar presupuesto</button></p>
    <p v-if="success" class="notice notice-success" role="status">{{ success }}</p>
    <section v-if="visiblePending.length" class="panel" aria-labelledby="pending-title"><h2 id="pending-title">Comandos pendientes</h2><p>El resultado de estos comandos todavía no se confirmó. Puedes reintentar el mismo envío.</p><ul><li v-for="command in visiblePending" :key="command.idempotencyKey"><span>{{ command.route.startsWith('/api/categories') ? 'Categoría' : command.route.startsWith('/api/budget-groups') ? 'Grupo' : command.route.includes('/close') ? 'Cierre' : 'Asignación' }} · {{ command.createdAt }}</span><button type="button" :disabled="busy" @click="retry(command)">Reintentar comando exacto</button><button type="button" :disabled="busy" @click="dismiss(command)">Ocultar aviso</button></li></ul></section>
    <div class="budget-tabs" role="tablist" aria-label="Área del presupuesto"><button id="planning-tab" type="button" role="tab" :aria-selected="tab === 'planning'" aria-controls="planning-panel" :tabindex="tab === 'planning' ? 0 : -1" @click="tab = 'planning'" @keydown.right.prevent="tab = 'categories'; ($event.currentTarget as HTMLElement).parentElement?.querySelector<HTMLElement>('#category-tab')?.focus()">Presupuesto</button><button id="category-tab" type="button" role="tab" :aria-selected="tab === 'categories'" aria-controls="category-panel" :tabindex="tab === 'categories' ? 0 : -1" @click="tab = 'categories'" @keydown.left.prevent="tab = 'planning'; ($event.currentTarget as HTMLElement).parentElement?.querySelector<HTMLElement>('#planning-tab')?.focus()">Administrar categorías</button></div>
    <section id="category-panel" v-show="tab === 'categories'" role="tabpanel" aria-labelledby="category-tab" class="admin-panel">
      <BudgetCategoryManager ref="categoryManager" :categories="categories" :groups="groups" :disabled="busy" :pending-routes="pending.map((item) => item.route)" :load-error="categoryLoadError" :refresh="refreshOrganization" :restore-pending="restorePending" :error-message="message" @busy-change="categoryBusy = $event" />
      <BudgetGroupManager :groups="groups" :disabled="busy" :pending-routes="pending.map((item) => item.route)" :refresh="refreshOrganization" :restore-pending="restorePending" @busy-change="groupBusy = $event" />
    </section>
    <p v-if="loading" role="status">Cargando presupuesto…</p>
    <section id="planning-panel" v-show="tab === 'planning'" role="tabpanel" aria-labelledby="planning-tab" class="planning-panel">
      <template v-if="budget">
        <BudgetAvailableSummary :budget="budget" />
        <p v-if="categoryLoadError" role="alert">{{ categoryLoadError }} <button type="button" @click="refreshOrganization().catch((error) => categoryLoadError = message(error, 'No fue posible actualizar la organización.'))">Actualizar organización</button></p>
        <BudgetPlanningTable :budget="budget" :groups="groups" :categories="categories" :assignments="assignmentWorkflow" :disabled="busy" :pending-routes="pending.map((item) => item.route)" :refresh="refreshOrganization" :restore-pending="restorePending" @details="categoryManager?.openCategory($event)" @busy-change="organizationBusy = $event" />
        <BudgetClosePanel v-if="!budget.closed" :preview="preview" :dispositions="dispositions" :disabled="busy" @load-preview="loadClosePreview" @close="closePeriod" @update-disposition="setDisposition" />
      </template>
    </section>
  </main>
</template>

<style scoped>
.budget-page, .planning-panel, .admin-panel { display: grid; gap: var(--island-gap); min-width: 0; }
.budget-page { padding-block: 4px 24px; }.page-heading { display: flex; justify-content: space-between; align-items: center; gap: 1rem; }.period-heading { display: flex; flex-wrap: wrap; align-items: center; gap: .65rem; }.page-heading h1 { margin: 0; font-size: 1.4rem; }.period-heading label { color: var(--muted); font-size: .9rem; }.period-heading input { min-height: 2.5rem; padding: .4rem .6rem; }
.budget-tabs { display: flex; flex-wrap: wrap; gap: .5rem; }.budget-tabs button { border: 1px solid var(--border); background: var(--surface); padding: .65rem 1rem; border-radius: var(--radius-sm); }.budget-tabs button[aria-selected=true] { border-color: var(--accent); color: var(--accent); font-weight: 700; }
.panel { background: var(--surface); border: 1px solid var(--border); border-radius: var(--island-radius); padding: var(--island-padding); }.notice { padding: .75rem; background: var(--warning-bg); border-radius: .4rem; }.notice-error { background: var(--danger-bg); }.notice-success { background: var(--success-bg); }.text-button { border: 0; background: none; text-decoration: underline; }button { min-height: 2.5rem; cursor: pointer; }button:disabled { opacity: .65; }
@media (max-width: 650px) { .page-heading { align-items: start; flex-direction: row; }.period-heading { gap: .4rem; }.period-heading h1 { width: 100%; } }
</style>
