<script setup lang="ts">
import type {
  TreeTableExpandedKeys,
  TreeTablePassThroughOptions,
} from "primevue/treetable";
import type { TreeNode } from "primevue/treenode";

import type { BudgetNodeType, BudgetTreeNode } from "@/features/budget/model/treeNodes";
import BaseButton from "@/common/components/base/BaseButton.vue";
import BaseColumn from "@/common/components/base/BaseColumn.vue";
import BaseTreeTable from "@/common/components/base/BaseTreeTable.vue";
import BudgetAssignedAmountCell from "@/features/budget/components/BudgetAssignedAmountCell.vue";
import BudgetMoneyCell from "@/features/budget/components/BudgetMoneyCell.vue";
import BudgetNodeNameCell from "@/features/budget/components/BudgetNodeNameCell.vue";

defineProps<{
  canDeleteNode: (node: BudgetTreeNode) => boolean;
  editingAssignedEnvelopeId: string;
  editingNameNodeId: string;
  isProtectedNode: (node: { key: string }) => boolean;
  isTreeEditing: boolean;
  nodes: TreeNode[];
  treeTablePassThrough: TreeTablePassThroughOptions;
}>();

defineEmits<{
  addCategory: [];
  addEnvelope: [subcategoryId: string];
  addSubcategory: [categoryId: string];
  completeName: [nodeType: BudgetNodeType, nodeId: string];
  deleteCategory: [categoryId: string];
  deleteEnvelope: [envelopeId: string];
  deleteSubcategory: [subcategoryId: string];
  editAssignedAmount: [envelopeId: string];
  editName: [nodeId: string];
  openGoal: [envelopeId: string];
  stopEditingAssignedAmount: [envelopeId: string];
  updateAssignedAmount: [envelopeId: string, amount: string];
  updateName: [nodeType: BudgetNodeType, nodeId: string, name: string];
}>();

const expandedKeys = defineModel<TreeTableExpandedKeys>("expandedKeys", {
  required: true,
});
</script>

<template>
  <BaseTreeTable
    v-model:expanded-keys="expandedKeys"
    class="budget-tree-table"
    :pt="treeTablePassThrough"
    :value="nodes"
  >
    <BaseColumn
      field="name"
      header="Sobre"
      expander
      sortable
      style="min-width: 18rem"
    >
      <template #header>
        <div class="name-column-header">
          <span>Sobre</span>
          <BaseButton
            v-if="isTreeEditing"
            size="sm"
            type="button"
            variant="secondary"
            @click.stop="$emit('addCategory')"
          >
            + Categoría
          </BaseButton>
        </div>
      </template>

      <template #body="{ node }">
        <BudgetNodeNameCell
          :can-delete="canDeleteNode(node)"
          :is-editing="isTreeEditing"
          :is-name-editing="editingNameNodeId === node.key"
          :is-protected="isProtectedNode(node)"
          :node="node"
          @add-envelope="$emit('addEnvelope', $event)"
          @add-subcategory="$emit('addSubcategory', $event)"
          @complete-name="
            (nodeType, nodeId) => $emit('completeName', nodeType, nodeId)
          "
          @delete-category="$emit('deleteCategory', $event)"
          @delete-envelope="$emit('deleteEnvelope', $event)"
          @delete-subcategory="$emit('deleteSubcategory', $event)"
          @edit-name="$emit('editName', $event)"
          @open-goal="$emit('openGoal', $event)"
          @update-name="
            (nodeType, nodeId, name) =>
              $emit('updateName', nodeType, nodeId, name)
          "
        />
      </template>
    </BaseColumn>

    <BaseColumn
      field="assignedAmount"
      header="Asignado"
      body-class="amount-column-body"
      header-class="amount-column-header"
      style="width: 12rem"
    >
      <template #body="{ node }">
        <BudgetAssignedAmountCell
          v-if="node.data.nodeType === 'envelope'"
          :amount="node.data.assignedAmount"
          :is-editing="editingAssignedEnvelopeId === node.key"
          :label="node.data.name"
          @blur="$emit('stopEditingAssignedAmount', node.key)"
          @edit="$emit('editAssignedAmount', node.key)"
          @update-amount="$emit('updateAssignedAmount', node.key, $event)"
        />
        <BudgetMoneyCell v-else :amount="node.data.assignedAmount" />
      </template>
    </BaseColumn>

    <BaseColumn
      field="spentAmount"
      header="Gastado"
      body-class="amount-column-body"
      header-class="amount-column-header"
      style="width: 12rem"
    >
      <template #body="{ node }">
        <BudgetMoneyCell :amount="node.data.spentAmount" signed />
      </template>
    </BaseColumn>

    <BaseColumn
      field="remainingAmount"
      header="Restante"
      body-class="amount-column-body"
      header-class="amount-column-header"
      style="width: 12rem"
    >
      <template #body="{ node }">
        <BudgetMoneyCell :amount="node.data.remainingAmount" toned />
      </template>
    </BaseColumn>

    <template #empty>Sin sobres para mostrar.</template>
  </BaseTreeTable>
</template>

<style scoped>
.budget-tree-table {
  font-size: var(--lu-font-size-md);
  font-weight: 400;
}

.budget-tree-table :deep(.p-treetable-thead > tr > th),
.budget-tree-table :deep(.p-treetable-tbody > tr > td) {
  font-size: inherit;
  font-weight: inherit;
}

.name-column-header {
  display: inline-flex;
  align-items: center;
  gap: var(--lu-space-2);
}

:deep(.p-treetable-thead > tr > th) {
  white-space: nowrap;
}

:deep(.amount-column-header),
:deep(.amount-column-body) {
  text-align: end;
}

:deep(.amount-column-header .p-treetable-column-header-content) {
  justify-content: flex-end;
}

:deep(.budget-row) {
  cursor: grab;
}

:deep(.budget-row--category > td) {
  background: var(--lu-color-primary-soft);
}

:deep(.budget-row--subcategory > td) {
  background: var(--lu-color-surface-muted);
}

:deep(.budget-row[data-dragging="true"] > td) {
  opacity: 0.55;
}

:deep(.budget-row[data-drag-over="true"] > td) {
  background: color-mix(in srgb, var(--lu-color-primary) 12%, transparent);
  color: var(--lu-color-text);
  box-shadow: inset 0 0 0 1px var(--lu-color-primary);
}

:deep(.budget-row[data-drag-edge="before"] > td) {
  box-shadow: inset 0 2px 0 var(--lu-color-primary);
}

:deep(.budget-row[data-drag-edge="after"] > td) {
  box-shadow: inset 0 -2px 0 var(--lu-color-primary);
}

:deep(.budget-row button),
:deep(.budget-row input) {
  cursor: auto;
}
</style>
