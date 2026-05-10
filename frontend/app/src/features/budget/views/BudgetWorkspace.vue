<script setup lang="ts">
import BaseBadge from "@/common/components/base/BaseBadge.vue";
import BaseButton from "@/common/components/base/BaseButton.vue";
import BaseCard from "@/common/components/base/BaseCard.vue";
import BudgetGoalDrawer from "@/features/budget/components/BudgetGoalDrawer.vue";
import BudgetSummaryBanner from "@/features/budget/components/BudgetSummaryBanner.vue";
import BudgetTreeTable from "@/features/budget/components/BudgetTreeTable.vue";
import { useBudgetWorkspace } from "@/features/budget/composables/useBudgetWorkspace";

const props = defineProps<{
  period: string;
}>();

const {
  currency,
  editingAssignedEnvelopeId,
  editingNameNodeId,
  expandedKeys,
  goalForm,
  incomeLeftToAssign,
  incomeLeftTone,
  isGoalDrawerOpen,
  isTreeEditing,
  nodes,
  selectedGoalEnvelope,
  suggestedGoalContribution,
  treeTablePassThrough,
  visibleEnvelopeCount,
  addCategory,
  addEnvelope,
  addSubcategory,
  archiveGoal,
  canDeleteNode,
  closeGoalDetails,
  completeEditingNodeName,
  completeGoal,
  deleteCategory,
  deleteEnvelope,
  deleteSubcategory,
  editAssignedAmount,
  editNodeName,
  isProtectedNode,
  openGoalDetails,
  saveGoalDetails,
  stopEditingAssignedAmount,
  toggleTreeEditing,
  updateEnvelopeAssignedAmountFromInput,
  updateNodeName,
} = useBudgetWorkspace(() => props.period);
</script>

<template>
  <BaseCard class="budget-panel">
    <template #header>
      <div class="budget-heading">
        <div>
          <h2 id="budget-heading">Presupuesto</h2>
          <span>Categorías, subcategorías y sobres</span>
        </div>

        <div class="budget-heading__actions">
          <BaseButton
            :aria-pressed="isTreeEditing"
            size="sm"
            type="button"
            variant="secondary"
            @click="toggleTreeEditing"
          >
            {{ isTreeEditing ? "Terminar edición" : "Editar árbol" }}
          </BaseButton>
          <BaseBadge>{{ visibleEnvelopeCount }} sobres</BaseBadge>
        </div>
      </div>
    </template>

    <section aria-labelledby="budget-heading">
      <BudgetSummaryBanner
        :amount="incomeLeftToAssign"
        :currency="currency"
        :tone="incomeLeftTone"
      />

      <BudgetTreeTable
        v-model:expanded-keys="expandedKeys"
        :can-delete-node="canDeleteNode"
        :editing-assigned-envelope-id="editingAssignedEnvelopeId"
        :editing-name-node-id="editingNameNodeId"
        :is-protected-node="isProtectedNode"
        :is-tree-editing="isTreeEditing"
        :nodes="nodes"
        :tree-table-pass-through="treeTablePassThrough"
        @add-category="addCategory"
        @add-envelope="addEnvelope"
        @add-subcategory="addSubcategory"
        @complete-name="completeEditingNodeName"
        @delete-category="deleteCategory"
        @delete-envelope="deleteEnvelope"
        @delete-subcategory="deleteSubcategory"
        @edit-assigned-amount="editAssignedAmount"
        @edit-name="editNodeName"
        @open-goal="openGoalDetails"
        @stop-editing-assigned-amount="stopEditingAssignedAmount"
        @update-assigned-amount="updateEnvelopeAssignedAmountFromInput"
        @update-name="updateNodeName"
      />
    </section>
  </BaseCard>

  <BudgetGoalDrawer
    v-if="isGoalDrawerOpen"
    v-model:deadline-date="goalForm.deadlineDate"
    v-model:target-amount="goalForm.targetAmount"
    :assigned-amount="selectedGoalEnvelope?.assignedAmount ?? '0'"
    :currency="currency"
    :envelope="selectedGoalEnvelope"
    :suggested-contribution="suggestedGoalContribution"
    @archive="archiveGoal"
    @close="closeGoalDetails"
    @complete="completeGoal"
    @save="saveGoalDetails"
  />
</template>

<style scoped>
.budget-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--lu-space-3);
  margin-block-end: 0;
}

.budget-heading h2 {
  margin: 0;
  font-size: var(--lu-font-size-md);
}

.budget-heading span {
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-sm);
}

.budget-heading__actions {
  display: flex;
  align-items: center;
  gap: var(--lu-space-2);
}

@media (max-width: 680px) {
  .budget-heading {
    align-items: stretch;
    flex-direction: column;
  }

  .budget-heading__actions {
    justify-content: space-between;
  }
}
</style>
