<script setup lang="ts">
import { useQuery } from "@pinia/colada";
import { computed, ref, watch, type VNode } from "vue";
import {
  draggable,
  dropTargetForElements,
} from "@atlaskit/pragmatic-drag-and-drop/element/adapter";
import Column from "primevue/column";
import type {
  TreeTableExpandedKeys,
  TreeTablePassThroughMethodOptions,
  TreeTablePassThroughOptions,
} from "primevue/treetable";
import TreeTable from "primevue/treetable";
import type { TreeNode } from "primevue/treenode";

import { getBudgetTree, getMovements, saveBudgetTree } from "@/api/ledger";
import {
  TRASH_CATEGORY_ID,
  TRASH_SUBCATEGORY_ID,
  applyBudgetTree,
  toBudgetTreeDto,
  useBudgetTree,
  type BudgetEnvelope,
} from "@/budget/budgetTree";
import BaseBadge from "@/components/base/BaseBadge.vue";
import BaseButton from "@/components/base/BaseButton.vue";
import BaseCard from "@/components/base/BaseCard.vue";
import BaseTextInput from "@/components/base/BaseTextInput.vue";
import { formatMoney } from "@/formatters";

type BudgetNodeType = "category" | "subcategory" | "envelope";

type BudgetNodeData = {
  name: string;
  nodeType: BudgetNodeType;
  assignedAmount: string;
  spentAmount: string;
  remainingAmount: string;
};

type BudgetSignedDirection = "in" | "out" | "none";

type BudgetSignedAmountParts = {
  direction: BudgetSignedDirection;
  number: string;
  prefix: string;
  text: string;
};

type EnvelopeDropInstruction = {
  targetEnvelopeId: string;
  edge: "before" | "after";
};

type SubcategoryDropInstruction = {
  targetSubcategoryId: string;
  edge: "before" | "after";
};

type CategoryDropInstruction = {
  targetCategoryId: string;
  edge: "before" | "after";
};

type BudgetTreeNode = TreeNode & {
  key: string;
  data: BudgetNodeData;
};

type BudgetRowPassThroughOptions = TreeTablePassThroughMethodOptions & {
  instance?: {
    node?: BudgetTreeNode;
  };
};

type DragElement = HTMLElement & {
  budgetDraggableCleanup?: () => void;
  budgetDraggableKey?: string;
  budgetDropTargetCleanup?: () => void;
  budgetDropTargetKey?: string;
  budgetRowRegistrationKey?: string;
};

const currency = "CLP";
const editingAssignedEnvelopeId = ref("");
const editingNameNodeId = ref("");
const isTreeEditing = ref(false);
const nextCategoryNumber = ref(3);
const nextSubcategoryNumber = ref(4);
const nextEnvelopeNumber = ref(5);
const { categories, subcategories, envelopes } = useBudgetTree();
const expandedKeys = ref<TreeTableExpandedKeys>({
  "cat-living": true,
  "cat-planning": true,
  "sub-home": true,
  "sub-food": true,
  "sub-reserve": true,
  [TRASH_CATEGORY_ID]: true,
  [TRASH_SUBCATEGORY_ID]: true,
});

const movementsQuery = useQuery({
  key: ["movements"],
  query: getMovements,
});
const budgetTreeQuery = useQuery({
  key: ["budget-tree"],
  query: getBudgetTree,
});

const movements = computed(() => movementsQuery.data.value ?? []);

watch(
  () => budgetTreeQuery.data.value,
  (tree) => {
    if (tree) {
      applyBudgetTree(tree);
      expandExistingBudgetTree();
    }
  },
  { immediate: true },
);

const spentByEnvelopeId = computed<Record<string, string>>(() => {
  const totals: Record<string, number> = {};

  movements.value.forEach((movement) => {
    if (!movement.envelopeId) {
      return;
    }

    totals[movement.envelopeId] =
      (totals[movement.envelopeId] ?? 0) + budgetMovementAmount(movement);
  });

  return Object.fromEntries(
    Object.entries(totals).map(([envelopeId, amount]) => [
      envelopeId,
      formatDecimal(amount),
    ]),
  );
});

function budgetMovementAmount(movement: {
  baseAmount: string;
  direction: "in" | "out";
}): number {
  const amount = parseAmount(movement.baseAmount);
  return movement.direction === "out" ? amount : -amount;
}

const budgetEnvelopes = computed<BudgetEnvelope[]>(() =>
  envelopes.value.map((envelope) => ({
    ...envelope,
    spentAmount: spentByEnvelopeId.value[envelope.id] ?? "0",
  })),
);

const nodes = computed<TreeNode[]>(() =>
  categories.value.map((category) => {
    const categorySubcategories = subcategories.value.filter(
      (subcategory) => subcategory.categoryId === category.id,
    );
    const categoryChildren = categorySubcategories.map((subcategory) => {
      const subcategoryEnvelopes = budgetEnvelopes.value.filter(
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
    });

    return {
      key: category.id,
      data: buildParentNodeData(
        "category",
        category.name,
        categoryChildren.map((subcategory) => subcategory.data),
      ),
      children: categoryChildren,
    };
  }),
);

const treeTablePassThrough: TreeTablePassThroughOptions = {
  row(options) {
    const node = (options as BudgetRowPassThroughOptions).instance?.node;

    if (!node) {
      return undefined;
    }

    return {
      class: ["budget-row", `budget-row--${node.data.nodeType}`],
      draggable: "true",
      "data-budget-node-id": node.key,
      "data-budget-node-type": node.data.nodeType,
      onVnodeMounted(vnode: VNode) {
        registerBudgetRow(getVnodeElement(vnode), node);
      },
      onVnodeUpdated(vnode: VNode) {
        registerBudgetRow(getVnodeElement(vnode), node);
      },
      onVnodeBeforeUnmount(vnode: VNode) {
        cleanupRowRegistration(getVnodeElement(vnode));
      },
    };
  },
};

const vAssignedInputFocus = {
  mounted(element: HTMLElement) {
    element.focus();

    if (element instanceof HTMLInputElement) {
      element.select();
    }
  },
};

const vNameInputFocus = {
  mounted(element: HTMLElement) {
    element.focus();

    if (element instanceof HTMLInputElement) {
      element.select();
    }
  },
};

function toggleTreeEditing() {
  isTreeEditing.value = !isTreeEditing.value;

  if (!isTreeEditing.value) {
    editingNameNodeId.value = "";
  }
}

async function persistBudgetTree() {
  const savedTree = await saveBudgetTree(toBudgetTreeDto());
  applyBudgetTree(savedTree);
}

function expandExistingBudgetTree() {
  expandedKeys.value = {
    ...expandedKeys.value,
    ...Object.fromEntries(
      categories.value.map((category) => [category.id, true]),
    ),
    ...Object.fromEntries(
      subcategories.value.map((subcategory) => [subcategory.id, true]),
    ),
  };
}

function addCategory() {
  if (!isTreeEditing.value) {
    return;
  }

  const categoryNumber = nextCategoryNumber.value;
  const categoryId = crypto.randomUUID();
  nextCategoryNumber.value = categoryNumber + 1;
  categories.value = [
    ...categories.value,
    { id: categoryId, name: `Nueva categoría ${categoryNumber}` },
  ];
  expandedKeys.value = {
    ...expandedKeys.value,
    [categoryId]: true,
  };
  editingNameNodeId.value = categoryId;
  void persistBudgetTree();
}

function addSubcategory(categoryId: string) {
  if (!isTreeEditing.value) {
    return;
  }

  const subcategoryNumber = nextSubcategoryNumber.value;
  const subcategoryId = crypto.randomUUID();
  nextSubcategoryNumber.value = subcategoryNumber + 1;
  subcategories.value = [
    ...subcategories.value,
    {
      id: subcategoryId,
      categoryId,
      name: `Nueva subcategoría ${subcategoryNumber}`,
    },
  ];
  expandSubcategoryParent(categoryId, subcategoryId);
  editingNameNodeId.value = subcategoryId;
  void persistBudgetTree();
}

function addEnvelope(subcategoryId: string) {
  if (!isTreeEditing.value) {
    return;
  }

  const envelopeNumber = nextEnvelopeNumber.value;
  const envelopeId = crypto.randomUUID();
  nextEnvelopeNumber.value = envelopeNumber + 1;
  envelopes.value = [
    ...envelopes.value,
    {
      id: envelopeId,
      subcategoryId,
      name: `Nuevo sobre ${envelopeNumber}`,
      assignedAmount: "0",
      spentAmount: "0",
    },
  ];
  expandEnvelopeParent(subcategoryId);
  editingNameNodeId.value = envelopeId;
  void persistBudgetTree();
}

function deleteCategory(categoryId: string) {
  if (!isTreeEditing.value) {
    return;
  }

  const removedSubcategoryIds = subcategories.value
    .filter((subcategory) => subcategory.categoryId === categoryId)
    .map((subcategory) => subcategory.id);
  const removedEnvelopeIds = envelopes.value
    .filter((envelope) =>
      removedSubcategoryIds.includes(envelope.subcategoryId),
    )
    .map((envelope) => envelope.id);
  categories.value = categories.value.filter(
    (category) => category.id !== categoryId,
  );
  subcategories.value = subcategories.value.filter(
    (subcategory) => subcategory.categoryId !== categoryId,
  );
  envelopes.value = envelopes.value.filter(
    (envelope) => !removedSubcategoryIds.includes(envelope.subcategoryId),
  );
  clearRemovedNodeEditing([
    categoryId,
    ...removedSubcategoryIds,
    ...removedEnvelopeIds,
  ]);
  void persistBudgetTree();
}

function deleteSubcategory(subcategoryId: string) {
  if (!isTreeEditing.value) {
    return;
  }

  const removedEnvelopeIds = envelopes.value
    .filter((envelope) => envelope.subcategoryId === subcategoryId)
    .map((envelope) => envelope.id);
  subcategories.value = subcategories.value.filter(
    (subcategory) => subcategory.id !== subcategoryId,
  );
  envelopes.value = envelopes.value.filter(
    (envelope) => envelope.subcategoryId !== subcategoryId,
  );
  clearRemovedNodeEditing([subcategoryId, ...removedEnvelopeIds]);
  void persistBudgetTree();
}

function deleteEnvelope(envelopeId: string) {
  if (!isTreeEditing.value) {
    return;
  }

  envelopes.value = envelopes.value.filter(
    (envelope) => envelope.id !== envelopeId,
  );
  clearRemovedNodeEditing([envelopeId]);
  void persistBudgetTree();
}

function editNodeName(nodeId: string) {
  if (!isTreeEditing.value) {
    return;
  }

  editingNameNodeId.value = nodeId;
}

function clearRemovedNodeEditing(nodeIds: string[]) {
  if (nodeIds.includes(editingNameNodeId.value)) {
    editingNameNodeId.value = "";
  }

  if (nodeIds.includes(editingAssignedEnvelopeId.value)) {
    editingAssignedEnvelopeId.value = "";
  }
}

function isTrashNode(node: { key: string }): boolean {
  return node.key === TRASH_CATEGORY_ID || node.key === TRASH_SUBCATEGORY_ID;
}

function updateNodeName(
  nodeType: BudgetNodeType,
  nodeId: string,
  name: string,
) {
  if (nodeType === "category") {
    categories.value = categories.value.map((category) =>
      category.id === nodeId ? { ...category, name } : category,
    );
    return;
  }

  if (nodeType === "subcategory") {
    subcategories.value = subcategories.value.map((subcategory) =>
      subcategory.id === nodeId ? { ...subcategory, name } : subcategory,
    );
    return;
  }

  envelopes.value = envelopes.value.map((envelope) =>
    envelope.id === nodeId ? { ...envelope, name } : envelope,
  );
}

function completeEditingNodeName(nodeType: BudgetNodeType, nodeId: string) {
  const nodeName = nodeNameById(nodeType, nodeId).trim();
  const fallbackName = defaultNodeName(nodeType);
  updateNodeName(nodeType, nodeId, nodeName === "" ? fallbackName : nodeName);

  if (editingNameNodeId.value === nodeId) {
    editingNameNodeId.value = "";
  }

  void persistBudgetTree();
}

function nodeNameById(nodeType: BudgetNodeType, nodeId: string): string {
  if (nodeType === "category") {
    return (
      categories.value.find((category) => category.id === nodeId)?.name ?? ""
    );
  }

  if (nodeType === "subcategory") {
    return (
      subcategories.value.find((subcategory) => subcategory.id === nodeId)
        ?.name ?? ""
    );
  }

  return envelopes.value.find((envelope) => envelope.id === nodeId)?.name ?? "";
}

function defaultNodeName(nodeType: BudgetNodeType): string {
  if (nodeType === "category") {
    return "Nueva categoría";
  }

  if (nodeType === "subcategory") {
    return "Nueva subcategoría";
  }

  return "Nuevo sobre";
}

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

function buildParentNodeData(
  nodeType: "category" | "subcategory",
  name: string,
  source: BudgetNodeData[],
): BudgetNodeData {
  const assignedAmount = sumNodeField(source, "assignedAmount");
  const spentAmount = sumNodeField(source, "spentAmount");

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

function getVnodeElement(vnode: VNode): DragElement | undefined {
  return vnode.el instanceof HTMLElement ? vnode.el : undefined;
}

function registerBudgetRow(
  element: DragElement | undefined,
  node: BudgetTreeNode,
) {
  if (!element) {
    return;
  }

  const registrationKey = `${node.data.nodeType}:${node.key}`;
  if (element.budgetRowRegistrationKey === registrationKey) {
    return;
  }

  cleanupRowRegistration(element);
  element.budgetRowRegistrationKey = registrationKey;

  if (node.data.nodeType === "category") {
    registerDraggableCategory(element, node.key);
    registerCategoryDropTarget(element, node.key);
    return;
  }

  if (node.data.nodeType === "subcategory") {
    registerDraggableSubcategory(element, node.key);
    registerSubcategoryDropTarget(element, node.key);
    return;
  }

  registerDraggableEnvelope(element, node.key);
  registerEnvelopeDropTarget(element, node.key);
}

function registerDraggableEnvelope(element: DragElement, envelopeId: string) {
  cleanupDraggableRegistration(element);
  element.budgetDraggableKey = envelopeId;
  element.budgetDraggableCleanup = draggable({
    element,
    canDrag({ input }) {
      return canStartRowDrag(input.clientX, input.clientY);
    },
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

function registerDraggableCategory(element: DragElement, categoryId: string) {
  cleanupDraggableRegistration(element);
  element.budgetDraggableKey = categoryId;
  element.budgetDraggableCleanup = draggable({
    element,
    canDrag({ input }) {
      return canStartRowDrag(input.clientX, input.clientY);
    },
    getInitialData() {
      return {
        type: "budget-category",
        categoryId,
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

function registerDraggableSubcategory(
  element: DragElement,
  subcategoryId: string,
) {
  cleanupDraggableRegistration(element);
  element.budgetDraggableKey = subcategoryId;
  element.budgetDraggableCleanup = draggable({
    element,
    canDrag({ input }) {
      return canStartRowDrag(input.clientX, input.clientY);
    },
    getInitialData() {
      return {
        type: "budget-subcategory",
        subcategoryId,
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

function canStartRowDrag(clientX: number, clientY: number): boolean {
  const target = document.elementFromPoint(clientX, clientY);
  return !target?.closest(
    "button, input, textarea, select, a, [contenteditable]",
  );
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
      return (
        source.data.type === "budget-envelope" ||
        (source.data.type === "budget-subcategory" &&
          source.data.subcategoryId !== subcategoryId)
      );
    },
    getData({ input }) {
      return {
        type: "budget-subcategory",
        subcategoryId,
        targetSubcategoryId: subcategoryId,
        edge: dropEdge(element, input.clientY),
      };
    },
    onDrag({ source, self }) {
      updateSubcategoryDropState(element, source.data.type, self.data.edge);
    },
    onDragEnter({ source, self }) {
      updateSubcategoryDropState(element, source.data.type, self.data.edge);
    },
    onDragLeave() {
      clearDropState(element);
    },
    onDrop({ source, self }) {
      clearDropState(element);
      if (typeof source.data.envelopeId === "string") {
        moveEnvelope(source.data.envelopeId, subcategoryId);
        return;
      }

      if (typeof source.data.subcategoryId !== "string") {
        return;
      }

      const instruction = parseSubcategoryDropInstruction(self.data);
      if (instruction) {
        moveSubcategoryNear(source.data.subcategoryId, instruction);
      }
    },
  });
}

function registerCategoryDropTarget(element: DragElement, categoryId: string) {
  cleanupDropTargetRegistration(element);
  element.budgetDropTargetKey = categoryId;
  element.budgetDropTargetCleanup = dropTargetForElements({
    element,
    canDrop({ source }) {
      return (
        source.data.type === "budget-subcategory" ||
        (source.data.type === "budget-category" &&
          source.data.categoryId !== categoryId)
      );
    },
    getData({ input }) {
      return {
        type: "budget-category",
        categoryId,
        targetCategoryId: categoryId,
        edge: dropEdge(element, input.clientY),
      };
    },
    onDrag({ source, self }) {
      updateCategoryDropState(element, source.data.type, self.data.edge);
    },
    onDragEnter({ source, self }) {
      updateCategoryDropState(element, source.data.type, self.data.edge);
    },
    onDragLeave() {
      clearDropState(element);
    },
    onDrop({ source, self }) {
      clearDropState(element);
      if (typeof source.data.subcategoryId === "string") {
        moveSubcategory(source.data.subcategoryId, categoryId);
        return;
      }

      if (typeof source.data.categoryId !== "string") {
        return;
      }

      const instruction = parseCategoryDropInstruction(self.data);
      if (instruction) {
        moveCategoryNear(source.data.categoryId, instruction);
      }
    },
  });
}

function updateSubcategoryDropState(
  element: HTMLElement,
  sourceType: unknown,
  edge: unknown,
) {
  if (sourceType === "budget-envelope") {
    delete element.dataset.dragEdge;
    element.dataset.dragOver = "true";
    return;
  }

  delete element.dataset.dragOver;
  element.dataset.dragEdge = edge === "before" ? "before" : "after";
}

function updateCategoryDropState(
  element: HTMLElement,
  sourceType: unknown,
  edge: unknown,
) {
  if (sourceType === "budget-subcategory") {
    delete element.dataset.dragEdge;
    element.dataset.dragOver = "true";
    return;
  }

  delete element.dataset.dragOver;
  element.dataset.dragEdge = edge === "before" ? "before" : "after";
}

function clearDropState(element: HTMLElement) {
  delete element.dataset.dragOver;
  delete element.dataset.dragEdge;
}

function cleanupRowRegistration(element: DragElement | undefined) {
  if (!element) {
    return;
  }

  cleanupDraggableRegistration(element);
  cleanupDropTargetRegistration(element);
  delete element.dataset.dragEdge;
  delete element.dataset.dragOver;
  delete element.dataset.dragging;
  delete element.budgetRowRegistrationKey;
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
  void persistBudgetTree();
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
  void persistBudgetTree();
}

function moveSubcategory(subcategoryId: string, targetCategoryId: string) {
  const draggedSubcategory = subcategories.value.find(
    (subcategory) => subcategory.id === subcategoryId,
  );
  const targetCategory = categories.value.find(
    (category) => category.id === targetCategoryId,
  );

  if (!draggedSubcategory || !targetCategory) {
    return;
  }

  const withoutDraggedSubcategory = subcategories.value.filter(
    (subcategory) => subcategory.id !== subcategoryId,
  );
  subcategories.value = [
    ...withoutDraggedSubcategory,
    { ...draggedSubcategory, categoryId: targetCategory.id },
  ];
  expandSubcategoryParent(targetCategory.id, draggedSubcategory.id);
  void persistBudgetTree();
}

function moveSubcategoryNear(
  subcategoryId: string,
  instruction: SubcategoryDropInstruction,
) {
  const draggedSubcategory = subcategories.value.find(
    (subcategory) => subcategory.id === subcategoryId,
  );
  const targetSubcategory = subcategories.value.find(
    (subcategory) => subcategory.id === instruction.targetSubcategoryId,
  );

  if (!draggedSubcategory || !targetSubcategory) {
    return;
  }

  const withoutDraggedSubcategory = subcategories.value.filter(
    (subcategory) => subcategory.id !== subcategoryId,
  );
  const targetIndex = withoutDraggedSubcategory.findIndex(
    (subcategory) => subcategory.id === instruction.targetSubcategoryId,
  );

  if (targetIndex < 0) {
    return;
  }

  const insertionIndex =
    instruction.edge === "before" ? targetIndex : targetIndex + 1;
  subcategories.value = [
    ...withoutDraggedSubcategory.slice(0, insertionIndex),
    { ...draggedSubcategory, categoryId: targetSubcategory.categoryId },
    ...withoutDraggedSubcategory.slice(insertionIndex),
  ];
  expandSubcategoryParent(targetSubcategory.categoryId, draggedSubcategory.id);
  void persistBudgetTree();
}

function moveCategoryNear(
  categoryId: string,
  instruction: CategoryDropInstruction,
) {
  const draggedCategory = categories.value.find(
    (category) => category.id === categoryId,
  );
  const withoutDraggedCategory = categories.value.filter(
    (category) => category.id !== categoryId,
  );
  const targetIndex = withoutDraggedCategory.findIndex(
    (category) => category.id === instruction.targetCategoryId,
  );

  if (!draggedCategory || targetIndex < 0) {
    return;
  }

  const insertionIndex =
    instruction.edge === "before" ? targetIndex : targetIndex + 1;
  categories.value = [
    ...withoutDraggedCategory.slice(0, insertionIndex),
    draggedCategory,
    ...withoutDraggedCategory.slice(insertionIndex),
  ];
  expandCategory(draggedCategory.id);
  void persistBudgetTree();
}

function updateEnvelopeAssignedAmount(
  envelopeId: string,
  assignedAmount: string,
) {
  envelopes.value = envelopes.value.map((envelope) =>
    envelope.id === envelopeId ? { ...envelope, assignedAmount } : envelope,
  );
}

function editAssignedAmount(envelopeId: string) {
  editingAssignedEnvelopeId.value = envelopeId;
}

function stopEditingAssignedAmount(envelopeId: string) {
  if (editingAssignedEnvelopeId.value === envelopeId) {
    editingAssignedEnvelopeId.value = "";
  }

  void persistBudgetTree();
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

function expandSubcategoryParent(categoryId: string, subcategoryId: string) {
  expandedKeys.value = {
    ...expandedKeys.value,
    [categoryId]: true,
    [subcategoryId]: true,
  };
}

function expandCategory(categoryId: string) {
  expandedKeys.value = {
    ...expandedKeys.value,
    [categoryId]: true,
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

function parseSubcategoryDropInstruction(
  data: Record<string | symbol, unknown>,
): SubcategoryDropInstruction | undefined {
  if (
    typeof data.targetSubcategoryId !== "string" ||
    (data.edge !== "before" && data.edge !== "after")
  ) {
    return undefined;
  }

  return {
    targetSubcategoryId: data.targetSubcategoryId,
    edge: data.edge,
  };
}

function parseCategoryDropInstruction(
  data: Record<string | symbol, unknown>,
): CategoryDropInstruction | undefined {
  if (
    typeof data.targetCategoryId !== "string" ||
    (data.edge !== "before" && data.edge !== "after")
  ) {
    return undefined;
  }

  return {
    targetCategoryId: data.targetCategoryId,
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

function signedSpentAmountParts(spentAmount: string): BudgetSignedAmountParts {
  const spentValue = parseAmount(spentAmount);
  const direction = spentDirection(spentValue);
  const sign = spentSign(direction);
  const formattedAmount = formatMoney(
    formatDecimal(Math.abs(spentValue)),
    currency,
  );
  const amountMatch = /^(\D*)(.*)$/.exec(formattedAmount);

  if (!amountMatch) {
    const text = `${sign}${formattedAmount}`;
    return {
      direction,
      number: formattedAmount,
      prefix: sign,
      text,
    };
  }

  const prefix = `${sign}${amountMatch[1]}`;
  const number = amountMatch[2];
  return {
    direction,
    number,
    prefix,
    text: `${prefix}${number}`,
  };
}

function spentDirection(spentValue: number): BudgetSignedDirection {
  if (spentValue > 0) {
    return "out";
  }

  if (spentValue < 0) {
    return "in";
  }

  return "none";
}

function spentSign(direction: BudgetSignedDirection): string {
  if (direction === "out") {
    return "-";
  }

  if (direction === "in") {
    return "+";
  }

  return "";
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

function sumNodeField(
  source: BudgetNodeData[],
  field: "assignedAmount" | "spentAmount",
): string {
  const total = source.reduce((sum, node) => sum + parseAmount(node[field]), 0);
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
          <BaseBadge>{{ envelopes.length }} sobres</BaseBadge>
        </div>
      </div>
    </template>

    <section aria-labelledby="budget-heading">
      <TreeTable
        v-model:expanded-keys="expandedKeys"
        class="budget-tree-table"
        :pt="treeTablePassThrough"
        :value="nodes"
      >
        <Column
          field="name"
          header="Sobre"
          expander
          sortable
          style="min-width: 18rem"
        >
          <template #header>
            <div class="name-column-header">
              <span>Sobre</span>
              <button
                v-if="isTreeEditing"
                class="add-category-header-button"
                type="button"
                @click.stop="addCategory"
              >
                + Categoría
              </button>
            </div>
          </template>

          <template #body="{ node }">
            <div class="node-name-cell">
              <BaseTextInput
                v-if="isTreeEditing && editingNameNodeId === node.key"
                v-name-input-focus
                :aria-label="`Nombre para ${node.data.name}`"
                class="node-name-input"
                :model-value="node.data.name"
                type="text"
                @blur="completeEditingNodeName(node.data.nodeType, node.key)"
                @keydown.enter="
                  completeEditingNodeName(node.data.nodeType, node.key)
                "
                @update:model-value="
                  (value) => updateNodeName(node.data.nodeType, node.key, value)
                "
              />
              <span
                v-else-if="node.data.nodeType === 'envelope'"
                class="envelope-label"
                :data-envelope-id="node.key"
              >
                <strong>{{ node.data.name }}</strong>
              </span>
              <span
                v-else-if="node.data.nodeType === 'subcategory'"
                class="group-label"
                data-level="subcategory"
                :data-subcategory-id="node.key"
              >
                <strong>{{ node.data.name }}</strong>
              </span>
              <span
                v-else
                class="group-label"
                data-level="category"
                :data-category-id="node.key"
              >
                {{ node.data.name }}
              </span>

              <button
                v-if="isTreeEditing && !isTrashNode(node)"
                class="rename-node-button"
                type="button"
                :aria-label="`Renombrar ${node.data.name}`"
                @click="editNodeName(node.key)"
              >
                Renombrar
              </button>
              <button
                v-if="
                  isTreeEditing &&
                  node.data.nodeType === 'category' &&
                  !isTrashNode(node)
                "
                class="add-child-button"
                type="button"
                :aria-label="`Agregar subcategoría en ${node.data.name}`"
                @click="addSubcategory(node.key)"
              >
                + Subcategoría
              </button>
              <button
                v-else-if="
                  isTreeEditing &&
                  node.data.nodeType === 'subcategory' &&
                  !isTrashNode(node)
                "
                class="add-child-button"
                type="button"
                :aria-label="`Agregar sobre en ${node.data.name}`"
                @click="addEnvelope(node.key)"
              >
                + Sobre
              </button>
              <button
                v-if="
                  isTreeEditing &&
                  node.data.nodeType === 'category' &&
                  !isTrashNode(node)
                "
                class="delete-node-button"
                type="button"
                :aria-label="`Eliminar categoría ${node.data.name}`"
                @click="deleteCategory(node.key)"
              >
                Eliminar
              </button>
              <button
                v-else-if="
                  isTreeEditing &&
                  node.data.nodeType === 'subcategory' &&
                  !isTrashNode(node)
                "
                class="delete-node-button"
                type="button"
                :aria-label="`Eliminar subcategoría ${node.data.name}`"
                @click="deleteSubcategory(node.key)"
              >
                Eliminar
              </button>
              <button
                v-else-if="isTreeEditing && node.data.nodeType === 'envelope'"
                class="delete-envelope-button"
                type="button"
                :aria-label="`Eliminar sobre ${node.data.name}`"
                @click="deleteEnvelope(node.key)"
              >
                Eliminar
              </button>
            </div>
          </template>
        </Column>

        <Column field="assignedAmount" header="Asignado" style="width: 12rem">
          <template #body="{ node }">
            <BaseTextInput
              v-if="
                node.data.nodeType === 'envelope' &&
                editingAssignedEnvelopeId === node.key
              "
              v-assigned-input-focus
              :aria-label="`Asignado para ${node.data.name}`"
              class="assigned-input"
              inputmode="decimal"
              :model-value="node.data.assignedAmount"
              type="text"
              @update:model-value="
                (value) => updateEnvelopeAssignedAmount(node.key, value)
              "
              @blur="stopEditingAssignedAmount(node.key)"
            />
            <button
              v-else-if="node.data.nodeType === 'envelope'"
              class="assigned-display numeric"
              type="button"
              :aria-label="`Editar asignado para ${node.data.name}`"
              @click="editAssignedAmount(node.key)"
            >
              {{ formatMoney(node.data.assignedAmount, currency) }}
            </button>
            <span v-else class="numeric">
              {{ formatMoney(node.data.assignedAmount, currency) }}
            </span>
          </template>
        </Column>

        <Column field="spentAmount" header="Gastado" style="width: 12rem">
          <template #body="{ node }">
            <span
              class="numeric"
              :aria-label="signedSpentAmountParts(node.data.spentAmount).text"
              :data-direction="
                signedSpentAmountParts(node.data.spentAmount).direction
              "
            >
              <span class="numeric__prefix">
                {{ signedSpentAmountParts(node.data.spentAmount).prefix }}
              </span>
              <span class="numeric__amount">
                {{ signedSpentAmountParts(node.data.spentAmount).number }}
              </span>
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

.budget-heading__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.node-name-cell {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  max-inline-size: 100%;
}

.name-column-header {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
}

.add-category-header-button {
  min-block-size: 1.75rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-secondary);
  color: var(--color-muted-foreground);
  cursor: pointer;
  padding: 0 var(--space-2);
  font-size: var(--font-size-xs);
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}

.add-category-header-button:hover,
.add-category-header-button:focus-visible {
  border-color: color-mix(in srgb, var(--color-primary) 34%, transparent);
  color: var(--color-primary-strong);
  outline: 0;
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

.node-name-input {
  min-inline-size: 10rem;
  max-inline-size: 16rem;
  min-block-size: 2.25rem;
}

.rename-node-button,
.add-child-button {
  min-block-size: 1.75rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-secondary);
  color: var(--color-muted-foreground);
  cursor: pointer;
  padding: 0 var(--space-2);
  font-size: var(--font-size-xs);
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}

.rename-node-button:hover,
.rename-node-button:focus-visible,
.add-child-button:hover,
.add-child-button:focus-visible {
  border-color: color-mix(in srgb, var(--color-primary) 34%, transparent);
  color: var(--color-primary-strong);
  outline: 0;
}

.delete-node-button,
.delete-envelope-button {
  min-block-size: 1.75rem;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-danger-strong);
  cursor: pointer;
  padding: 0 var(--space-2);
  font-size: var(--font-size-xs);
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}

.delete-node-button:hover,
.delete-node-button:focus-visible,
.delete-envelope-button:hover,
.delete-envelope-button:focus-visible {
  border-color: color-mix(in srgb, var(--color-danger) 34%, transparent);
  background: color-mix(in srgb, var(--color-danger) 9%, transparent);
  outline: 0;
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

.numeric[data-direction="in"] {
  color: var(--color-primary-strong);
}

.numeric[data-direction="out"] {
  color: var(--color-danger-strong);
}

.assigned-display {
  border: 0;
  background: transparent;
  cursor: text;
  padding: 0;
  text-align: end;
}

.assigned-display:hover,
.assigned-display:focus-visible {
  color: var(--color-primary);
  outline: 0;
}

.assigned-display:focus-visible {
  text-decoration: underline;
  text-underline-offset: 0.2em;
}

.assigned-input {
  max-inline-size: 10rem;
  min-block-size: 2.25rem;
  text-align: end;
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

:deep(.budget-row) {
  cursor: grab;
}

:deep(.budget-row--category > td) {
  background: color-mix(in srgb, var(--color-primary) 9%, transparent);
}

:deep(.budget-row--subcategory > td) {
  background: color-mix(in srgb, var(--color-secondary) 78%, transparent);
}

:deep(.budget-row[data-dragging="true"] > td) {
  opacity: 0.55;
}

:deep(.budget-row[data-drag-over="true"] > td) {
  background: color-mix(in srgb, var(--color-primary) 12%, transparent);
  color: var(--color-primary-strong);
  box-shadow: inset 0 0 0 1px var(--color-border-strong);
}

:deep(.budget-row[data-drag-edge="before"] > td) {
  box-shadow: inset 0 2px 0 var(--color-primary);
}

:deep(.budget-row[data-drag-edge="after"] > td) {
  box-shadow: inset 0 -2px 0 var(--color-primary);
}

:deep(.budget-row button),
:deep(.budget-row input) {
  cursor: auto;
}

@media (max-width: 680px) {
  .budget-heading {
    align-items: stretch;
    flex-direction: column;
  }

  .budget-heading__actions {
    justify-content: space-between;
  }

  .node-name-cell {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
