<script setup lang="ts">
import { computed } from 'vue'
import type { AvailableToBudgetBreakdown, BudgetSnapshot } from '../api/contracts'
import { budgetValueReasons, budgetValueText } from '../budget-model'

const props = defineProps<{ budget: BudgetSnapshot }>()
const labels: Array<[keyof AvailableToBudgetBreakdown, string]> = [
  ['prior_unassigned', 'Sin asignar del período anterior'], ['income', 'Ingresos'],
  ['openings', 'Saldos iniciales'], ['transfers', 'Transferencias presupuestarias'],
  ['valuation', 'Ajustes de valoración'], ['corrections', 'Correcciones'],
  ['category_releases', 'Saldos liberados al cierre'], ['category_rollover', 'Arrastre entre categorías'],
  ['assignments', 'Menos: asignaciones'],
]
const components = computed(() => labels.map(([key, label]) => ({ key, label, value: props.budget.available_to_budget_breakdown?.[key] })))
</script>

<template>
  <section class="panel available-card" aria-labelledby="available-title">
    <div class="available-heading"><div><p class="context">{{ budget.period }} · CLP</p><h2 id="available-title">Disponible para presupuestar</h2></div><strong class="numeric available-total">{{ budgetValueText(budget.ready_to_assign) }}</strong></div>
    <ul v-if="budgetValueReasons(budget.ready_to_assign).length"><li v-for="reason in budgetValueReasons(budget.ready_to_assign)" :key="reason">{{ reason }}</li></ul>
    <details class="calculation" open><summary>Cómo se calcula</summary><p>Saldo previo + financiamiento + liberaciones − asignaciones. Arrastre entre categorías: CLP 0.</p>
      <dl class="breakdown"><div v-for="item in components" :key="item.key"><dt>{{ item.label }}</dt><dd class="numeric">{{ budgetValueText(item.value) }}<small v-for="reason in budgetValueReasons(item.value)" :key="reason">{{ reason }}</small></dd></div></dl>
    </details>
  </section>
</template>

<style scoped>
.panel { background: var(--surface); border: 1px solid var(--border); border-radius: var(--island-radius); padding: var(--island-padding); box-shadow: var(--shadow-island); }
.available-card { border-top: 4px solid var(--accent); padding: .75rem; }.available-heading { display: flex; flex-wrap: wrap; gap: 1rem; justify-content: space-between; align-items: center; }.available-heading h2 { margin: .2rem 0; font-size: 1.1rem; }.context { margin: 0; color: var(--muted); font-size: .85rem; }.available-total { font-size: clamp(1.5rem, 3vw, 2.3rem); }
.calculation { margin-top: .35rem; }.calculation summary { cursor: pointer; font-weight: 600; padding-block: .25rem; }.calculation > p { margin-block: .35rem; }.breakdown { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 0 1.5rem; margin-block: .35rem; }.breakdown > div { display: flex; justify-content: space-between; gap: .7rem; padding-block: .2rem; border-bottom: 1px solid var(--border); }.breakdown dd { margin: 0; text-align: right; white-space: nowrap; }.breakdown small { display: block; color: var(--muted); white-space: normal; }
@media (max-width: 520px) { .breakdown { grid-template-columns: 1fr; } }
</style>
