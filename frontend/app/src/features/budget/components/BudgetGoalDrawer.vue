<script setup lang="ts">
import type { BudgetEnvelope } from "@/features/budget/model/budgetTree";
import BaseButton from "@/common/components/base/BaseButton.vue";
import BaseField from "@/common/components/base/BaseField.vue";
import BaseTextInput from "@/common/components/base/BaseTextInput.vue";
import { formatMoney } from "@/common/formatters";

defineProps<{
  assignedAmount: string;
  currency: string;
  envelope: BudgetEnvelope | undefined;
  suggestedContribution: string;
}>();

defineEmits<{
  archive: [];
  close: [];
  complete: [];
  save: [];
}>();

const targetAmount = defineModel<string>("targetAmount", { required: true });
const deadlineDate = defineModel<string>("deadlineDate", { required: true });
</script>

<template>
  <Teleport to="body">
    <div
      class="goal-drawer-backdrop"
      role="presentation"
      @click.self="$emit('close')"
    >
      <aside
        aria-labelledby="goal-drawer-title"
        aria-modal="true"
        class="goal-drawer"
        role="dialog"
      >
        <header class="goal-drawer__header">
          <div>
            <p class="eyebrow">Meta</p>
            <h2 id="goal-drawer-title">
              {{ envelope?.name ?? "Meta" }}
            </h2>
          </div>
          <BaseButton type="button" variant="secondary" @click="$emit('close')">
            Cerrar
          </BaseButton>
        </header>

        <form class="goal-form" @submit.prevent="$emit('save')">
          <dl class="goal-summary">
            <div>
              <dt>Asignado este periodo</dt>
              <dd>
                {{ formatMoney(assignedAmount, currency) }}
              </dd>
            </div>
            <div>
              <dt>Sugerido mensual</dt>
              <dd>{{ formatMoney(suggestedContribution, currency) }}</dd>
            </div>
            <div>
              <dt>Sobre vinculado</dt>
              <dd>{{ envelope?.name ?? "Sin sobre" }}</dd>
            </div>
          </dl>

          <BaseField input-id="goal-target" label="Monto objetivo">
            <BaseTextInput
              id="goal-target"
              v-model="targetAmount"
              inputmode="decimal"
              pattern="\d+(\.\d+)?"
              required
            />
          </BaseField>

          <BaseField input-id="goal-deadline" label="Fecha objetivo">
            <BaseTextInput
              id="goal-deadline"
              v-model="deadlineDate"
              type="date"
            />
          </BaseField>

          <div class="goal-form__actions">
            <BaseButton
              type="button"
              variant="secondary"
              @click="$emit('complete')"
            >
              Completar
            </BaseButton>
            <BaseButton
              type="button"
              variant="danger"
              @click="$emit('archive')"
            >
              Archivar
            </BaseButton>
            <BaseButton type="submit">Guardar</BaseButton>
          </div>
        </form>
      </aside>
    </div>
  </Teleport>
</template>

<style>
.goal-drawer-backdrop {
  position: fixed;
  z-index: var(--lu-z-drawer);
  inset: 0;
  display: flex;
  justify-content: flex-end;
  background: var(--lu-color-backdrop);
}

.goal-drawer {
  display: grid;
  align-content: start;
  gap: var(--lu-space-5);
  inline-size: min(var(--lu-size-drawer), 100%);
  block-size: 100dvh;
  overflow-y: auto;
  border-inline-start: 1px solid var(--lu-color-border);
  background: var(--lu-color-surface);
  padding: var(--lu-space-5);
  box-shadow: var(--lu-shadow-lg);
}

.goal-drawer__header {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: var(--lu-space-4);
}

.goal-drawer__header h2 {
  margin: 0;
  font-size: var(--lu-font-size-xl);
}

.goal-drawer .eyebrow {
  margin: 0 0 var(--lu-space-1);
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-xs);
  font-weight: 800;
  text-transform: uppercase;
}

.goal-form {
  display: grid;
  gap: var(--lu-space-3);
}

.goal-summary {
  display: grid;
  gap: var(--lu-space-2);
  margin: 0;
}

.goal-summary div {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--lu-space-3);
  border-block-end: 1px solid var(--lu-color-border);
  padding-block-end: var(--lu-space-2);
}

.goal-summary dt {
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-sm);
  font-weight: 700;
}

.goal-summary dd {
  margin: 0;
  color: var(--lu-color-text);
  font-weight: 800;
  text-align: end;
}

.goal-form__actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--lu-space-2);
  padding-block-start: var(--lu-space-2);
}

@media (max-width: 680px) {
  .goal-drawer {
    inline-size: 100%;
  }
}
</style>
