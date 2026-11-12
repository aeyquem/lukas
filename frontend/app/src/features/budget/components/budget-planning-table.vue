<script setup lang="ts">
import { computed, ref, toRef, watch } from 'vue'
import type { BudgetCategory, BudgetGroup, BudgetSnapshot, ExpenseCategory } from '../api/contracts'
import { budgetValueReasons, budgetValueText, categoryOverused, groupedBudgetSections, usagePercent } from '../budget-model'
import { useBudgetOrganization } from '../composables/use-budget-organization'
import type { useBudgetAssignments } from '../composables/use-budget-assignments'
import BudgetAssignmentCell from './budget-assignment-cell.vue'
import { formatMoney } from '../../../shared/formatting/money'

const props = defineProps<{
  budget: BudgetSnapshot
  groups: readonly BudgetGroup[]
  categories: readonly ExpenseCategory[]
  assignments: ReturnType<typeof useBudgetAssignments>
  disabled: boolean
  pendingRoutes: readonly string[]
  refresh: () => Promise<void>
  restorePending: () => void
}>()
const emit = defineEmits<{ details: [categoryId: string]; busyChange: [value: boolean] }>()
const organization = useBudgetOrganization({ groups: toRef(props, 'groups'), categories: toRef(props, 'categories'), refresh: props.refresh, restorePending: props.restorePending })
const { editing, saving, error, notice } = organization
watch(saving, (value) => emit('busyChange', value))
const sections = computed(() => groupedBudgetSections(props.budget.categories, props.groups))
const historical = computed(() => [...(props.budget.archived_categories ?? []), ...props.budget.categories.filter((item) => item.archived)])
const dragging = ref<{ kind: 'group' | 'category'; id: string } | null>(null)
const locked = computed(() => props.disabled || saving.value || props.pendingRoutes.some((route) => route.startsWith('/api/categories') || route.startsWith('/api/budget-groups')))
function assignmentPending(id: string): boolean { return props.pendingRoutes.includes(`/api/budgets/${props.budget.period}/assignments/${id}`) }
const dropTarget = ref<string | null>(null)
function endDrag(): void { dragging.value = null; dropTarget.value = null }
function dragOver(event: DragEvent, groupId: string): void {
  if (!dragging.value || locked.value) return
  event.preventDefault()
  dropTarget.value = groupId
  if (event.dataTransfer) event.dataTransfer.dropEffect = 'move'
}
function leaveGroup(event: DragEvent): void {
  if (event.relatedTarget instanceof Node && (event.currentTarget as HTMLElement).contains(event.relatedTarget)) return
  dropTarget.value = null
}
function reorder(ids: string[], sourceId: string, targetId: string): string[] {
  const result = ids.filter((id) => id !== sourceId)
  result.splice(Math.max(0, result.indexOf(targetId)), 0, sourceId)
  return result
}
async function moveGroup(id: string, direction: number): Promise<void> {
  if (locked.value || !editing.value) return
  const ids = sections.value.map((section) => section.group.id)
  const index = ids.indexOf(id); const target = ids[index + direction]
  if (!target) return
  ;[ids[index], ids[index + direction]] = [ids[index + direction], ids[index]]
  await organization.reorderGroups(ids)
}
async function moveCategory(id: string, groupId: string, direction: number): Promise<void> {
  if (locked.value) return
  const ids = sections.value.find((section) => section.group.id === groupId)?.categories.map((item) => item.id) ?? []
  const index = ids.indexOf(id)
  if (!ids[index + direction]) return
  ;[ids[index], ids[index + direction]] = [ids[index + direction], ids[index]]
  await organization.reorderCategories(groupId, ids)
}
async function moveToGroup(id: string, groupId: string): Promise<void> {
  if (locked.value) return
  const ids = sections.value.find((section) => section.group.id === groupId)?.categories.map((item) => item.id).filter((item) => item !== id) ?? []
  await organization.reorderCategories(groupId, [...ids, id])
}
function startDrag(event: DragEvent, kind: 'group' | 'category', id: string): void {
  if (locked.value || (kind === 'group' && !editing.value)) { event.preventDefault(); return }
  dragging.value = { kind, id }
  event.dataTransfer?.setData('text/plain', id)
  if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move'
}
async function dropOnGroup(id: string): Promise<void> {
  const source = dragging.value; endDrag()
  if (!source || locked.value) return
  if (source.kind === 'category') { await moveToGroup(source.id, id); return }
  if (editing.value && source.id !== id) await organization.reorderGroups(reorder(sections.value.map((item) => item.group.id), source.id, id))
}
async function dropOnCategory(category: BudgetCategory): Promise<void> {
  const source = dragging.value; endDrag()
  if (!source || source.kind !== 'category' || source.id === category.id || locked.value) return
  const ids = sections.value.find((item) => item.group.id === category.group_id)?.categories.map((item) => item.id) ?? []
  await organization.reorderCategories(category.group_id, reorder(ids, source.id, category.id))
}
</script>

<template>
  <section class="panel" aria-labelledby="planning-title">
    <header class="planning-heading"><h2 id="planning-title">Presupuesto</h2><button type="button" :disabled="locked" :aria-pressed="editing" @click="editing = !editing; endDrag()">{{ editing ? 'Terminar edición' : 'Editar organización' }}</button></header>
    <p v-if="error" role="alert">{{ error }}</p><p v-if="notice" role="status">{{ notice }}</p>
    <p class="table-help">Arrastra una categoría al grupo deseado. Haz clic en su asignado para editarlo.</p>
    <p v-if="editing">Usa Subir, Bajar y Mover al grupo, o arrastra las asas de los grupos. Cada cambio se guarda antes de actualizar los valores.</p>
    <p v-if="budget.closed">Período cerrado: las correcciones requieren un motivo. El asignado original se conserva como referencia.</p>
    <div class="table-scroll"><table aria-labelledby="planning-title"><thead><tr><th scope="col">Categoría</th><th scope="col" class="amount-heading">Asignado <small>CLP · editable</small></th><th scope="col" class="amount-heading">Usado <small>CLP</small></th><th scope="col" class="amount-heading">Disponible <small>CLP</small></th><th scope="col">Detalles</th></tr></thead>
      <tbody v-for="(section, groupIndex) in sections" :key="section.group.id" :data-group-id="section.group.id" :class="{ 'drop-target': dropTarget === section.group.id }" @dragover="dragOver($event, section.group.id)" @dragleave="leaveGroup" @drop.prevent="dropOnGroup(section.group.id)">
        <tr class="group-heading"><th scope="rowgroup" colspan="5"><div class="group-controls"><button v-if="editing" type="button" class="drag-handle" :aria-label="`Arrastrar grupo ${section.group.name}`" :draggable="editing && !locked" :disabled="locked" @dragstart="startDrag($event, 'group', section.group.id)" @dragend="endDrag">↕</button><span>{{ section.group.name }}</span><template v-if="editing"><button type="button" :aria-label="`Subir grupo ${section.group.name}`" :disabled="locked || groupIndex === 0" @click="moveGroup(section.group.id, -1)">Subir</button><button type="button" :aria-label="`Bajar grupo ${section.group.name}`" :disabled="locked || groupIndex === sections.length - 1" @click="moveGroup(section.group.id, 1)">Bajar</button></template></div></th></tr>
        <tr v-for="(category, categoryIndex) in section.categories" :key="category.id" :data-category-id="category.id" @drop.stop.prevent="dropOnCategory(category)">
          <th scope="row"><div class="category-name" :draggable="!locked" title="Arrastrar categoría a un grupo" @dragstart="startDrag($event, 'category', category.id)" @dragend="endDrag"><span class="drag-grip" aria-hidden="true">⠿</span><span>{{ category.name }}</span></div>
            <div class="usage-indicator" :class="{ 'is-overused': categoryOverused(category) }" role="img" :aria-label="usagePercent(category) === null ? `Uso de ${category.name} incompleto` : categoryOverused(category) ? `${category.name} sobrepasó su asignación` : `${category.name}: ${usagePercent(category)} por ciento utilizado`"><span v-if="usagePercent(category) !== null" :style="{ width: `${usagePercent(category)}%` }"></span></div><small class="detail">{{ usagePercent(category) === null ? 'Uso incompleto' : categoryOverused(category) ? 'Sobrepasado' : `${usagePercent(category)}% utilizado` }}</small>
            <div v-if="editing" class="category-moves"><button type="button" :aria-label="`Subir categoría ${category.name}`" :disabled="locked || categoryIndex === 0" @click="moveCategory(category.id, section.group.id, -1)">Subir</button><button type="button" :aria-label="`Bajar categoría ${category.name}`" :disabled="locked || categoryIndex === section.categories.length - 1" @click="moveCategory(category.id, section.group.id, 1)">Bajar</button><label :for="`move-${category.id}`" class="sr-only">Mover {{ category.name }} al grupo</label><select :id="`move-${category.id}`" :disabled="locked" :value="section.group.id" @change="moveToGroup(category.id, ($event.target as HTMLSelectElement).value)"><option v-for="target in sections" :key="target.group.id" :value="target.group.id">{{ target.group.name }}</option></select></div>
          </th>
          <td class="numeric assignment-cell"><BudgetAssignmentCell :key="`${budget.period}:${category.id}`" :category="category" :closed="!!budget.closed" :assignments="assignments" :disabled="disabled" :pending="assignmentPending(category.id)" /></td>
          <td class="numeric">{{ budgetValueText(category.activity) }}<small v-for="reason in budgetValueReasons(category.activity)" :key="reason" class="detail">{{ reason }}</small></td><td class="numeric">{{ budgetValueText(category.remaining) }}<small v-for="reason in budgetValueReasons(category.remaining)" :key="reason" class="detail">{{ reason }}</small></td><td><button type="button" :disabled="disabled" :aria-label="`Detalles de ${category.name}`" @click="emit('details', category.id)">Detalles</button></td>
        </tr>
      </tbody>
    </table></div>
    <p v-if="!sections.some((section) => section.categories.length)">No hay categorías activas. Añade una en Administrar categorías.</p>
    <div class="uncategorized"><h3>Actividad sin categoría</h3><strong class="numeric">{{ budgetValueText(budget.uncategorized_activity) }}</strong><p v-for="reason in budgetValueReasons(budget.uncategorized_activity)" :key="reason">{{ reason }}</p></div>
    <details v-if="historical.length" class="archived-history"><summary>Categorías archivadas con historia</summary><ul><li v-for="category in historical" :key="category.id"><strong>{{ category.name }}</strong> · Asignado {{ formatMoney(category.assigned, 'CLP') }} · Usado {{ budgetValueText(category.activity) }} · Restante {{ budgetValueText(category.remaining) }} <button type="button" :aria-label="`Detalles de ${category.name}`" @click="emit('details', category.id)">Detalles</button></li></ul></details>
  </section>
</template>

<style scoped>
.panel { background: var(--surface); border: 1px solid var(--border); min-width: 0; border-radius: var(--island-radius); padding: var(--island-padding); box-shadow: var(--shadow-island); }.planning-heading { display: flex; align-items: center; flex-wrap: wrap; gap: .7rem; justify-content: space-between; }.planning-heading h2 { margin: .25rem 0; }.table-scroll { overflow-x: auto; min-width: 0; }table { width: 100%; border-collapse: collapse; }th,td { padding: .75rem .6rem; border-bottom: 1px solid var(--border); text-align: left; vertical-align: top; }thead th { white-space: nowrap; color: var(--text); background: var(--surface-soft); border-bottom: 2px solid var(--border-strong); font-weight: 700; }thead small { display: block; color: var(--muted); font-size: .75rem; font-weight: 400; }.amount-heading,td.numeric { text-align: right; }.assignment-cell { min-width: 16rem; }.table-help { color: var(--muted); font-size: .9rem; }.drop-target { outline: 2px solid var(--accent); outline-offset: -2px; }.drop-target .group-heading th { color: var(--on-accent); background: var(--accent); }.category-name[draggable="true"] { cursor: grab; }.category-name[draggable="true"]:active { cursor: grabbing; }.drag-grip { color: var(--muted); user-select: none; }.group-heading th {
  background: var(--accent-soft);
  color: var(--text);
  border-top: .6rem solid var(--surface);
  border-bottom: 1px solid var(--border-strong);
  box-shadow: inset 4px 0 var(--accent);
  padding: .85rem 1rem;
  font-size: 1rem;
  font-weight: 700;
}.group-controls,.category-name,.category-moves { display: flex; align-items: center; flex-wrap: wrap; gap: .4rem; }.category-moves { margin-top: .6rem; }.detail { display: block; margin-top: .3rem; color: var(--muted); font-size: .8rem; font-weight: 400; }.usage-indicator { width: min(150px, 100%); height: 6px; margin-top: .55rem; overflow: hidden; border-radius: 999px; background: var(--border); }.usage-indicator span { display: block; height: 100%; background: var(--accent); }.usage-indicator.is-overused span { background: var(--danger); }.uncategorized { margin-top: 1rem; }.uncategorized h3 { margin-bottom: .4rem; }.archived-history { margin-top: 1rem; }.archived-history li { padding-block: .4rem; }.drag-handle { cursor: grab; }.sr-only { position: absolute; inset-inline-start: 0; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }button { min-height: 2.5rem; }button:disabled { opacity: .6; }
</style>
