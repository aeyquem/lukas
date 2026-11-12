<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { BudgetGroup } from '../api/contracts'
import { groupWriteIntent, useBudgetWriteMutation } from '../api/mutations'
const props = defineProps<{ groups: readonly BudgetGroup[]; disabled: boolean; pendingRoutes: readonly string[]; refresh: () => Promise<void>; restorePending: () => void }>()
const emit = defineEmits<{ busyChange: [value: boolean] }>()
const newName = ref('')
const drafts = ref<Record<string, { name: string; revision: number }>>({})
const error = ref('')
const notice = ref('')
const saving = ref(false)
const write = useBudgetWriteMutation()
const orderedGroups = computed(() => [...props.groups].sort((a, b) => a.sort_order - b.sort_order || a.id.localeCompare(b.id)))
watch(saving, (value) => emit('busyChange', value))
function pending(id: string | null): boolean { return props.pendingRoutes.includes(id ? `/api/budget-groups/${id}` : '/api/budget-groups') }
function edit(group: BudgetGroup, name: string): void { drafts.value[group.id] = { name, revision: drafts.value[group.id]?.revision ?? group.revision } }
async function save(group: BudgetGroup | null, archived?: boolean): Promise<void> {
  if (props.disabled || saving.value || pending(group?.id ?? null)) return
  const name = (group ? drafts.value[group.id]?.name ?? group.name : newName.value).trim()
  if (!name) { error.value = 'Escribe un nombre para el grupo.'; return }
  saving.value = true; error.value = ''; notice.value = ''
  try {
    await write.mutateAsync(groupWriteIntent(group?.id ?? null, { name, sort_order: group?.sort_order ?? Math.min(32767, Math.max(0, ...props.groups.map((item) => item.sort_order)) + 1), ...(group ? { archived: archived ?? group.archived, expected_revision: drafts.value[group.id]?.revision ?? group.revision } : {}) }))
    if (group) delete drafts.value[group.id]; else newName.value = ''
    notice.value = 'Grupo guardado.'
    await props.refresh()
  } catch (cause) { error.value = cause instanceof Error ? cause.message : 'No fue posible guardar el grupo.' }
  finally { props.restorePending(); saving.value = false }
}
function review(group: BudgetGroup): void { if (drafts.value[group.id]) drafts.value[group.id].revision = group.revision }
</script>
<template>
  <section class="panel" aria-labelledby="groups-title"><h2 id="groups-title">Grupos</h2>
    <p v-if="error" role="alert">{{ error }}</p><p v-if="notice" role="status">{{ notice }}</p>
    <form class="group-row" @submit.prevent="save(null)"><label>Nombre del grupo<input v-model="newName" :disabled="disabled || saving || pending(null)" required maxlength="80"></label><button :disabled="disabled || saving || pending(null)">Añadir grupo</button></form>
    <ul class="group-list"><li v-for="group in orderedGroups" :key="group.id" class="group-row"><label><span class="sr-only">Nombre del grupo {{ group.name }}</span><input :value="drafts[group.id]?.name ?? group.name" :disabled="disabled || saving || pending(group.id)" :aria-label="`Nombre del grupo ${group.name}`" maxlength="80" @input="edit(group, ($event.target as HTMLInputElement).value)"></label><span>{{ group.archived ? 'Archivado' : 'Activo' }}</span><button type="button" :disabled="disabled || saving || pending(group.id)" :aria-label="`Guardar grupo ${group.name}`" @click="save(group)">Guardar</button><button type="button" :disabled="disabled || saving || pending(group.id)" :aria-label="`${group.archived ? 'Restaurar' : 'Archivar'} grupo ${group.name}`" @click="save(group, !group.archived)">{{ group.archived ? 'Restaurar' : 'Archivar' }}</button><button v-if="drafts[group.id] && drafts[group.id].revision !== group.revision" type="button" :disabled="disabled || saving" @click="review(group)">Revisé el grupo vigente {{ group.name }}</button></li></ul>
    <p>Para cambiar el orden, activa Editar organización en Presupuesto. Un grupo debe quedar sin categorías activas antes de archivarlo.</p>
  </section>
</template>
<style scoped>
.panel { background: var(--surface); border: 1px solid var(--border); border-radius: var(--island-radius); padding: var(--island-padding); }.group-row { display: flex; align-items: end; flex-wrap: wrap; gap: .6rem; padding-block: .6rem; }.group-row label { display: grid; gap: .3rem; }.group-row input { padding: .5rem; max-width: 100%; }.group-list { list-style: none; padding: 0; }.group-list li { border-top: 1px solid var(--border); }.sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }button { min-height: 2.5rem; }
</style>
