<script setup lang="ts">
import type { BudgetClosePreview } from '../api/contracts'
import { closeCategoryLabel } from '../budget-model'

defineProps<{
  preview: BudgetClosePreview | null
  dispositions: Record<string, 'carry' | 'release'>
  disabled: boolean
}>()
const emit = defineEmits<{
  loadPreview: []
  close: []
  updateDisposition: [categoryId: string, disposition: 'carry' | 'release']
}>()
</script>

<template>
  <section class="panel close-panel" aria-labelledby="close-title">
    <h2 id="close-title">Cerrar período</h2>
    <p>Antes de cerrar, revisa cada saldo restante y elige si llevarlo al siguiente período o liberarlo.</p>
    <button type="button" class="button-secondary" @click="emit('loadPreview')">Ver vista previa del cierre</button>
    <div v-if="preview" class="close-preview">
      <p v-if="!preview.close.eligible" class="notice" role="status">{{ preview.close.reason || 'El período todavía no se puede cerrar.' }}</p>
      <fieldset v-for="item in preview.close.categories" :key="item.category_id">
        <legend>{{ closeCategoryLabel(preview, item.category_id) }}</legend>
        <label><input type="radio" :name="`disposition-${item.category_id}`" value="carry" :checked="(dispositions[item.category_id] ?? 'carry') === 'carry'" @change="emit('updateDisposition', item.category_id, 'carry')"> Llevar saldo</label>
        <label><input type="radio" :name="`disposition-${item.category_id}`" value="release" :checked="dispositions[item.category_id] === 'release'" @change="emit('updateDisposition', item.category_id, 'release')"> Liberar saldo</label>
      </fieldset>
      <button type="button" :disabled="!preview.close.eligible || disabled" @click="emit('close')">Confirmar cierre</button>
    </div>
  </section>
</template>

<style scoped>
.panel { background: var(--surface); border: 1px solid var(--border); min-width: 0; border-radius: var(--island-radius); padding: var(--island-padding); box-shadow: var(--shadow-island); }
.notice { padding: .75rem; background: var(--warning-bg); border-radius: .4rem; }
.close-preview { margin-top: 1rem; }.close-preview fieldset { display: flex; gap: 1rem; margin: .75rem 0; }.close-preview legend { font-weight: 600; }
button { min-height: 2.5rem; cursor: pointer; } button:disabled { cursor: wait; opacity: .65; }
@media (max-width: 650px) { .close-preview fieldset { flex-direction: column; } }
</style>
