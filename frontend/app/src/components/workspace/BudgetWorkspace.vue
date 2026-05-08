<script setup lang="ts">
import { computed, ref } from "vue";
import {
  draggable,
  dropTargetForElements,
} from "@atlaskit/pragmatic-drag-and-drop/element/adapter";
import Column from "primevue/column";
import type { TreeTableExpandedKeys } from "primevue/treetable";
import TreeTable from "primevue/treetable";
import type { TreeNode } from "primevue/treenode";

import BaseBadge from "@/components/base/BaseBadge.vue";
import BaseCard from "@/components/base/BaseCard.vue";
import { formatMoney } from "@/formatters";

type BudgetNodeType = "category" | "subcategory" | "envelope";

type BudgetNodeData = {
  name: string;
  nodeType: BudgetNodeType;
  assignedAmount: string;
  spentAmount: string;
  remainingAmount: string;
};

type BudgetCategory = {
  id: string;
  name: string;
};

type BudgetSubcategory = {
  id: string;
  categoryId: string;
  name: string;
};

type BudgetEnvelope = {
  id: string;
  subcategoryId: string;
  name: string;
  assignedAmount: string;
  spentAmount: string;
};

type EnvelopeDropInstruction = {
  targetEnvelopeId: string;
  edge: "before" | "after";
};

type DragElement = HTMLElement & {
  budgetDraggableCleanup?: () => void;
  budgetDraggableKey?: string;
  budgetDropTargetCleanup?: () => void;
  budgetDropTargetKey?: string;
};

const currency = "CLP";
const expandedKeys = ref<TreeTableExpandedKeys>({
  "cat-living": true,
  "cat-planning": true,
  "sub-home": true,
  "sub-food": true,
  "sub-reserve": true,
});

const categories = ref<BudgetCategory[]>([
  { id: "cat-living", name: "Vida diaria" },
  { id: "cat-planning", name: "Planificación" },
]);

const subcategories = ref<BudgetSubcategory[]>([
  { id: "sub-home", categoryId: "cat-living", name: "Casa" },
  { id: "sub-food", categoryId: "cat-living", name: "Comida" },
  { id: "sub-reserve", categoryId: "cat-planning", name: "Reservas" },
]);

const envelopes = ref<BudgetEnvelope[]>([
  {
    id: "env-rent",
    subcategoryId: "sub-home",
    name: "Arriendo",
    assignedAmount: "450000",
    spentAmount: "0",
  },
  {
    id: "env-market",
    subcategoryId: "sub-food",
    name: "Supermercado",
    assignedAmount: "180000",
    spentAmount: "64250",
  },
  {
    id: "env-emergency",
    subcategoryId: "sub-reserve",
    name: "Emergencias",
    assignedAmount: "100000",
    spentAmount: "0",
  },
  {
    id: "env-vacation",
    subcategoryId: "sub-reserve",
    name: "Vacaciones",
    assignedAmount: "80000",
    spentAmount: "0",
  },
]);

const nodes = computed<TreeNode[]>(() =>
  categories.value.map((category) => {
    const categorySubcategories = subcategories.value.filter(
      (subcategory) => subcategory.categoryId === category.id,
    );
    const categoryEnvelopes = envelopes.value.filter((envelope) =>
      categorySubcategories.some(
        (subcategory) => subcategory.id === envelope.subcategoryId,
      ),
    );

    return {
      key: category.id,
      data: buildNodeData("category", category.name, categoryEnvelopes),
      children: categorySubcategories.map((subcategory) => {
        const subcategoryEnvelopes = envelopes.value.filter(
          (envelope) => envelope.subcategoryId === subcategory.id,
        );

        return {
          key: subcategory.id,
          data: buildNodeData(
            "subcategory",
            subcategory.name,
            subcategoryEnvelopes,
          ),
          children: subcategoryEnvelopes.map((envelope) => ({
            key: envelope.id,
            data: {
              name: envelope.name,
              nodeType: "envelope",
              assignedAmount: envelope.assignedAmount,
              spentAmount: envelope.spentAmount,
              remainingAmount: remainingAmount(envelope),
            } satisfies BudgetNodeData,
          })),
        };
      }),
    };
  }),
);

const vDraggableEnvelope = {
  mounted(element: DragElement, binding: { value: string }) {
    registerDraggableEnvelope(element, binding.value);
  },
  updated(element: DragElement, binding: { value: string }) {
    if (element.budgetDraggableKey !== binding.value) {
      registerDraggableEnvelope(element, binding.value);
    }
  },
  unmounted(element: DragElement) {
    cleanupDraggableRegistration(element);
  },
};

const vSubcategoryDropTarget = {
  mounted(element: DragElement, binding: { value: string }) {
    registerSubcategoryDropTarget(element, binding.value);
  },
  updated(element: DragElement, binding: { value: string }) {
    if (element.budgetDropTargetKey !== binding.value) {
      registerSubcategoryDropTarget(element, binding.value);
    }
  },
  unmounted(element: DragElement) {
    cleanupDropTargetRegistration(element);
  },
};

const vEnvelopeDropTarget = {
  mounted(element: DragElement, binding: { value: string }) {
    registerEnvelopeDropTarget(element, binding.value);
  },
  updated(element: DragElement, binding: { value: string }) {
    if (element.budgetDropTargetKey !== binding.value) {
      registerEnvelopeDropTarget(element, binding.value);
    }
  },
  unmounted(element: DragElement) {
    cleanupDropTargetRegistration(element);
  },
};

function buildNodeData(
  nodeType: BudgetNodeType,
  name: string,
  source: BudgetEnvelope[],
): BudgetNodeData {
  const assignedAmount = sumEnvelopeField(source, "assignedAmount");
  const spentAmount = sumEnvelopeField(source, "spentAmount");

  return {
    name,
    nodeType,
    assignedAmount,
    spentAmount,
    remainingAmount: formatDecimal(
      parseAmount(assignedAmount) - parseAmount(spentAmount),
    ),
  };
}

function registerDraggableEnvelope(element: DragElement, envelopeId: string) {
  cleanupDraggableRegistration(element);
  element.budgetDraggableKey = envelopeId;
  element.budgetDraggableCleanup = draggable({
    element,
    getInitialData() {
      return {
        type: "budget-envelope",
        envelopeId,
      };
    },
    onDragStart() {
      element.dataset.dragging = "true";
    },
    onDrop() {
      delete element.dataset.dragging;
    },
  });
}

function registerEnvelopeDropTarget(element: DragElement, envelopeId: string) {
  cleanupDropTargetRegistration(element);
  element.budgetDropTargetKey = envelopeId;
  element.budgetDropTargetCleanup = dropTargetForElements({
    element,
    canDrop({ source }) {
      return (
        source.data.type === "budget-envelope" &&
        source.data.envelopeId !== envelopeId
      );
    },
    getData({ input }) {
      return {
        type: "budget-envelope-placement",
        targetEnvelopeId: envelopeId,
        edge: dropEdge(element, input.clientY),
      };
    },
    onDrag({ self }) {
      element.dataset.dragEdge = String(self.data.edge);
    },
    onDragEnter({ self }) {
      element.dataset.dragEdge = String(self.data.edge);
    },
    onDragLeave() {
      delete element.dataset.dragEdge;
    },
    onDrop({ source, self }) {
      delete element.dataset.dragEdge;
      if (typeof source.data.envelopeId !== "string") {
        return;
      }

      const instruction = parseEnvelopeDropInstruction(self.data);
      if (instruction) {
        moveEnvelopeNear(source.data.envelopeId, instruction);
      }
    },
  });
}

function registerSubcategoryDropTarget(
  element: DragElement,
  subcategoryId: string,
) {
  cleanupDropTargetRegistration(element);
  element.budgetDropTargetKey = subcategoryId;
  element.budgetDropTargetCleanup = dropTargetForElements({
    element,
    canDrop({ source }) {
      return source.data.type === "budget-envelope";
    },
    getData() {
      return {
        type: "budget-subcategory",
        subcategoryId,
      };
    },
    onDragEnter() {
      element.dataset.dragOver = "true";
    },
    onDragLeave() {
      delete element.dataset.dragOver;
    },
    onDrop({ source }) {
      delete element.dataset.dragOver;
      if (typeof source.data.envelopeId === "string") {
        moveEnvelope(source.data.envelopeId, subcategoryId);
      }
    },
  });
}

function cleanupDraggableRegistration(element: DragElement) {
  element.budgetDraggableCleanup?.();
  delete element.budgetDraggableCleanup;
  delete element.budgetDraggableKey;
}

function cleanupDropTargetRegistration(element: DragElement) {
  element.budgetDropTargetCleanup?.();
  delete element.budgetDropTargetCleanup;
  delete element.budgetDropTargetKey;
}

function moveEnvelope(envelopeId: string, targetSubcategoryId: string) {
  const targetSubcategory = subcategories.value.find(
    (subcategory) => subcategory.id === targetSubcategoryId,
  );

  if (!targetSubcategory) {
    return;
  }

  envelopes.value = envelopes.value.map((envelope) =>
    envelope.id === envelopeId
      ? { ...envelope, subcategoryId: targetSubcategoryId }
      : envelope,
  );
  expandEnvelopeParent(targetSubcategory.id);
}

function moveEnvelopeNear(
  envelopeId: string,
  instruction: EnvelopeDropInstruction,
) {
  const draggedEnvelope = envelopes.value.find(
    (envelope) => envelope.id === envelopeId,
  );
  const targetEnvelope = envelopes.value.find(
    (envelope) => envelope.id === instruction.targetEnvelopeId,
  );

  if (!draggedEnvelope || !targetEnvelope) {
    return;
  }

  const withoutDraggedEnvelope = envelopes.value.filter(
    (envelope) => envelope.id !== envelopeId,
  );
  const targetIndex = withoutDraggedEnvelope.findIndex(
    (envelope) => envelope.id === instruction.targetEnvelopeId,
  );

  if (targetIndex < 0) {
    return;
  }

  const insertionIndex =
    instruction.edge === "before" ? targetIndex : targetIndex + 1;
  envelopes.value = [
    ...withoutDraggedEnvelope.slice(0, insertionIndex),
    { ...draggedEnvelope, subcategoryId: targetEnvelope.subcategoryId },
    ...withoutDraggedEnvelope.slice(insertionIndex),
  ];
  expandEnvelopeParent(targetEnvelope.subcategoryId);
}

function expandEnvelopeParent(subcategoryId: string) {
  const subcategory = subcategories.value.find(
    (candidate) => candidate.id === subcategoryId,
  );

  if (!subcategory) {
    return;
  }

  expandedKeys.value = {
    ...expandedKeys.value,
    [subcategory.categoryId]: true,
    [subcategory.id]: true,
  };
}

function parseEnvelopeDropInstruction(
  data: Record<string | symbol, unknown>,
): EnvelopeDropInstruction | undefined {
  if (
    typeof data.targetEnvelopeId !== "string" ||
    (data.edge !== "before" && data.edge !== "after")
  ) {
    return undefined;
  }

  return {
    targetEnvelopeId: data.targetEnvelopeId,
    edge: data.edge,
  };
}

function dropEdge(element: HTMLElement, clientY: number): "before" | "after" {
  const bounds = element.getBoundingClientRect();
  return clientY < bounds.top + bounds.height / 2 ? "before" : "after";
}

function amountTone(amount: string): "positive" | "negative" {
  return Number.parseFloat(amount) >= 0 ? "positive" : "negative";
}

function sumEnvelopeField(
  source: BudgetEnvelope[],
  field: "assignedAmount" | "spentAmount",
): string {
  const total = source.reduce(
    (sum, envelope) => sum + parseAmount(envelope[field]),
    0,
  );
  return formatDecimal(total);
}

function remainingAmount(envelope: BudgetEnvelope): string {
  return formatDecimal(
    parseAmount(envelope.assignedAmount) - parseAmount(envelope.spentAmount),
  );
}

function parseAmount(amount: string): number {
  const value = Number.parseFloat(amount);
  return Number.isFinite(value) ? value : 0;
}

function formatDecimal(amount: number): string {
  return amount.toFixed(0);
}
</script>

<template>
  <BaseCard class="budget-panel">
    <template #header>
      <div class="budget-heading">
        <div>
          <h2 id="budget-heading">Presupuesto</h2>
          <span>Categorías, subcategorías y sobres</span>
        </div>

        <BaseBadge>{{ envelopes.length }} sobres</BaseBadge>
      </div>
    </template>

    <section aria-labelledby="budget-heading">
      <TreeTable
        v-model:expanded-keys="expandedKeys"
        class="budget-tree-table"
        :value="nodes"
      >
        <Column
          field="name"
          header="Sobre"
          expander
          sortable
          style="min-width: 18rem"
        >
          <template #body="{ node }">
            <span
              v-if="node.data.nodeType === 'envelope'"
              v-draggable-envelope="node.key"
              v-envelope-drop-target="node.key"
              class="envelope-label"
              :data-envelope-id="node.key"
              draggable="true"
            >
              <span class="drag-handle" aria-hidden="true">::</span>
              <strong>{{ node.data.name }}</strong>
            </span>
            <span
              v-else-if="node.data.nodeType === 'subcategory'"
              v-subcategory-drop-target="node.key"
              class="group-label drop-target"
              data-level="subcategory"
              :data-subcategory-id="node.key"
            >
              {{ node.data.name }}
            </span>
            <span v-else class="group-label" data-level="category">
              {{ node.data.name }}
            </span>
          </template>
        </Column>

        <Column field="assignedAmount" header="Asignado" style="width: 12rem">
          <template #body="{ node }">
            <span class="numeric">
              {{ formatMoney(node.data.assignedAmount, currency) }}
            </span>
          </template>
        </Column>

        <Column field="spentAmount" header="Gastado" style="width: 12rem">
          <template #body="{ node }">
            <span class="numeric">
              {{ formatMoney(node.data.spentAmount, currency) }}
            </span>
          </template>
        </Column>

        <Column field="remainingAmount" header="Restante" style="width: 12rem">
          <template #body="{ node }">
            <span
              class="numeric"
              :data-tone="amountTone(node.data.remainingAmount)"
            >
              {{ formatMoney(node.data.remainingAmount, currency) }}
            </span>
          </template>
        </Column>

        <template #empty>Sin sobres para mostrar.</template>
      </TreeTable>
    </section>
  </BaseCard>
</template>

<style scoped>
.budget-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-block-end: 0;
}

.budget-heading h2 {
  margin: 0;
  color: var(--color-primary-strong);
  font-size: 1rem;
}

.budget-heading span {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-sm);
}

.group-label,
.envelope-label {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
}

.group-label {
  color: var(--color-primary-strong);
  font-weight: 800;
}

.group-label[data-level="subcategory"] {
  color: var(--color-muted-foreground);
}

.drop-target {
  border-radius: var(--radius-md);
  padding: var(--space-1) var(--space-2);
}

.drop-target[data-drag-over="true"] {
  background: color-mix(in srgb, var(--color-primary) 12%, transparent);
  color: var(--color-primary-strong);
  box-shadow: inset 0 0 0 1px var(--color-border-strong);
}

.envelope-label {
  border-radius: var(--radius-md);
  padding: var(--space-1) var(--space-2);
}

.envelope-label[data-dragging="true"] {
  opacity: 0.55;
}

.envelope-label[data-drag-edge="before"] {
  box-shadow: inset 0 2px 0 var(--color-primary);
}

.envelope-label[data-drag-edge="after"] {
  box-shadow: inset 0 -2px 0 var(--color-primary);
}

.drag-handle {
  color: var(--color-muted-foreground);
  cursor: grab;
  font-family: var(--font-mono);
  font-weight: 800;
  letter-spacing: 0;
}

.numeric {
  color: var(--color-primary-strong);
  font-family: var(--font-mono);
  font-weight: 700;
  white-space: nowrap;
}

.numeric[data-tone="negative"] {
  color: var(--color-danger-strong);
}

:deep(.p-treetable-table) {
  font-size: var(--font-size-sm);
}

:deep(.p-treetable-column-title) {
  font-size: var(--font-size-sm);
}

:deep(.p-treetable-thead > tr > th) {
  white-space: nowrap;
}

@media (max-width: 680px) {
  .budget-heading {
    align-items: stretch;
    flex-direction: column;
  }
}
</style>
