<script setup lang="ts">
import { useQuery } from "@pinia/colada";
import { computed, reactive, watch } from "vue";

import { getBudgetTree, saveBudgetTree } from "@/api/ledger";
import {
  GOALS_SUBCATEGORY_ID,
  applyBudgetTree,
  toBudgetTreeDto,
  useBudgetTree,
} from "@/budget/budgetTree";
import BaseBadge from "@/components/base/BaseBadge.vue";
import BaseButton from "@/components/base/BaseButton.vue";
import BaseCard from "@/components/base/BaseCard.vue";
import BaseField from "@/components/base/BaseField.vue";
import BaseTextInput from "@/components/base/BaseTextInput.vue";
import { formatMoney } from "@/formatters";

const emit = defineEmits<{
  openBudget: [];
}>();

const currency = "CLP";
const { envelopes, goals } = useBudgetTree();
const form = reactive({
  name: "",
  targetAmount: "",
  deadlineDate: "",
});

const budgetTreeQuery = useQuery({
  key: ["budget-tree"],
  query: getBudgetTree,
});

const goalCards = computed(() =>
  goals.value
    .filter((goal) => goal.status !== "archived")
    .map((goal) => {
      const envelope = envelopes.value.find(
        (candidate) => candidate.id === goal.envelopeId,
      );
      return {
        goal,
        envelopeName: envelope?.name ?? "Sobre no encontrado",
        assignedAmount: envelope?.assignedAmount ?? "0",
        progress: progressPercent(envelope?.assignedAmount ?? "0", goal.targetAmount),
        warning: isBelowSuggestedPace(
          envelope?.assignedAmount ?? "0",
          goal.targetAmount,
          goal.deadlineDate,
        ),
      };
    }),
);
const activeGoalCount = computed(() => goalCards.value.length);

watch(
  () => budgetTreeQuery.data.value,
  (tree) => {
    if (tree) {
      applyBudgetTree(tree);
    }
  },
  { immediate: true },
);

async function createGoal() {
  const name = form.name.trim();
  const targetAmount = normalizedAmount(form.targetAmount);

  if (name === "" || parseAmount(targetAmount) <= 0) {
    return;
  }

  const envelopeId = crypto.randomUUID();
  envelopes.value = [
    ...envelopes.value,
    {
      id: envelopeId,
      subcategoryId: GOALS_SUBCATEGORY_ID,
      name,
      assignedAmount: "0",
      spentAmount: "0",
    },
  ];
  goals.value = [
    ...goals.value,
    {
      id: crypto.randomUUID(),
      envelopeId,
      targetAmount,
      deadlineDate: form.deadlineDate,
      status: "active",
    },
  ];

  await persistBudgetTree();
  resetForm();
  emit("openBudget");
}

async function persistBudgetTree() {
  const savedTree = await saveBudgetTree(toBudgetTreeDto());
  applyBudgetTree(savedTree);
}

function resetForm() {
  form.name = "";
  form.targetAmount = "";
  form.deadlineDate = "";
}

function progressPercent(assignedAmount: string, targetAmount: string): number {
  const target = parseAmount(targetAmount);
  if (target <= 0) {
    return 0;
  }

  return Math.min(100, Math.round((parseAmount(assignedAmount) / target) * 100));
}

function isBelowSuggestedPace(
  assignedAmount: string,
  targetAmount: string,
  deadlineDate: string,
): boolean {
  if (deadlineDate === "") {
    return false;
  }

  return parseAmount(assignedAmount) < suggestedContribution(targetAmount, deadlineDate);
}

function suggestedContribution(targetAmount: string, deadlineDate: string): number {
  const months = monthsUntilDeadline(deadlineDate);
  return Math.ceil(parseAmount(targetAmount) / months);
}

function monthsUntilDeadline(deadlineDate: string): number {
  const deadline = new Date(`${deadlineDate}T00:00:00`);
  if (Number.isNaN(deadline.getTime())) {
    return 1;
  }

  const today = new Date();
  const monthDelta =
    (deadline.getFullYear() - today.getFullYear()) * 12 +
    (deadline.getMonth() - today.getMonth()) +
    1;
  return Math.max(1, monthDelta);
}

function parseAmount(amount: string): number {
  const value = Number.parseFloat(amount);
  return Number.isFinite(value) ? value : 0;
}

function normalizedAmount(amount: string): string {
  return Math.max(0, parseAmount(amount)).toFixed(0);
}
</script>

<template>
  <BaseCard class="goals-panel">
    <template #header>
      <div class="goals-heading">
        <div>
          <h2 id="goals-heading">Metas</h2>
          <span>Seguimiento y ritmo de ahorro</span>
        </div>
        <BaseBadge>{{ activeGoalCount }} activas</BaseBadge>
      </div>
    </template>

    <section class="goals-layout" aria-labelledby="goals-heading">
      <form class="goal-create-form" @submit.prevent="createGoal">
        <BaseField input-id="new-goal-name" label="Nombre">
          <BaseTextInput
            id="new-goal-name"
            v-model="form.name"
            maxlength="120"
            required
          />
        </BaseField>

        <BaseField input-id="new-goal-target" label="Monto objetivo">
          <BaseTextInput
            id="new-goal-target"
            v-model="form.targetAmount"
            inputmode="decimal"
            pattern="\d+(\.\d+)?"
            required
          />
        </BaseField>

        <BaseField input-id="new-goal-deadline" label="Fecha objetivo">
          <BaseTextInput
            id="new-goal-deadline"
            v-model="form.deadlineDate"
            type="date"
          />
        </BaseField>

        <BaseButton type="submit">Crear meta</BaseButton>
      </form>

      <div class="goals-list" aria-label="Metas activas">
        <article
          v-for="card in goalCards"
          :key="card.goal.id"
          class="goal-card"
        >
          <header class="goal-card__header">
            <div>
              <h3>{{ card.envelopeName }}</h3>
              <span>
                {{
                  card.goal.deadlineDate === ""
                    ? "Sin fecha objetivo"
                    : card.goal.deadlineDate
                }}
              </span>
            </div>
            <BaseBadge :tone="card.warning ? 'warning' : 'success'">
              {{ card.progress }}%
            </BaseBadge>
          </header>

          <div class="goal-progress" aria-hidden="true">
            <span :style="{ inlineSize: `${card.progress}%` }"></span>
          </div>

          <dl class="goal-card__numbers">
            <div>
              <dt>Asignado</dt>
              <dd>{{ formatMoney(card.assignedAmount, currency) }}</dd>
            </div>
            <div>
              <dt>Objetivo</dt>
              <dd>{{ formatMoney(card.goal.targetAmount, currency) }}</dd>
            </div>
          </dl>

          <p v-if="card.warning" class="goal-warning">
            Asignación bajo el ritmo sugerido.
          </p>
        </article>

        <p v-if="goalCards.length === 0" class="goals-empty">
          No hay metas activas.
        </p>
      </div>
    </section>
  </BaseCard>
</template>

<style scoped>
.goals-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.goals-heading h2,
.goal-card h3 {
  margin: 0;
  color: var(--color-primary-strong);
  font-size: 1rem;
}

.goals-heading span,
.goal-card span,
.goals-empty,
.goal-warning {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-sm);
}

.goals-layout {
  display: grid;
  grid-template-columns: minmax(16rem, 22rem) minmax(0, 1fr);
  gap: var(--space-5);
}

.goal-create-form {
  align-content: start;
  display: grid;
  gap: var(--space-3);
}

.goals-list {
  display: grid;
  gap: var(--space-3);
}

.goal-card {
  display: grid;
  gap: var(--space-3);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-4);
}

.goal-card__header,
.goal-card__numbers {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: var(--space-3);
}

.goal-progress {
  overflow: hidden;
  block-size: 0.5rem;
  border-radius: var(--radius-round);
  background: var(--color-secondary);
}

.goal-progress span {
  display: block;
  block-size: 100%;
  border-radius: inherit;
  background: var(--color-primary);
}

.goal-card__numbers {
  margin: 0;
}

.goal-card__numbers dt {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-xs);
  font-weight: 800;
  text-transform: uppercase;
}

.goal-card__numbers dd {
  margin: var(--space-1) 0 0;
  color: var(--color-primary-strong);
  font-family: var(--font-mono);
  font-weight: 800;
}

.goal-warning {
  margin: 0;
  color: var(--color-warning);
  font-weight: 700;
}

.goals-empty {
  margin: 0;
}

@media (max-width: 820px) {
  .goals-layout {
    grid-template-columns: 1fr;
  }
}
</style>
