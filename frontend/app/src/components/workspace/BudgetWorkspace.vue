<script setup lang="ts">
import { useQuery } from "@pinia/colada";
import { computed, ref, watch, type VNode } from "vue";
import {
  draggable,
  dropTargetForElements,
} from "@atlaskit/pragmatic-drag-and-drop/element/adapter";
import type {
  TreeTableExpandedKeys,
  TreeTablePassThroughMethodOptions,
  TreeTablePassThroughOptions,
} from "primevue/treetable";
import type { TreeNode } from "primevue/treenode";

import { getBudgetTree, saveBudgetTree } from "@/api/ledger";
import {
  INCOME_CATEGORY_ID,
  INCOME_ENVELOPE_ID,
  INCOME_SUBCATEGORY_ID,
  TRASH_CATEGORY_ID,
  TRASH_SUBCATEGORY_ID,
  applyBudgetTree,
  isTrashEnvelope,
  toBudgetTreeDto,
  useBudgetTree,
  type BudgetEnvelope,
} from "@/budget/budgetTree";
import BaseBadge from "@/components/base/BaseBadge.vue";
import BaseButton from "@/components/base/BaseButton.vue";
import BaseCard from "@/components/base/BaseCard.vue";
import BaseColumn from "@/components/base/BaseColumn.vue";
import BaseField from "@/components/base/BaseField.vue";
import BaseTextInput from "@/components/base/BaseTextInput.vue";
import BaseTreeTable from "@/components/base/BaseTreeTable.vue";
import { formatMoney } from "@/formatters";

type BudgetNodeType = "category" | "subcategory" | "envelope";

type BudgetNodeData = {
  name: string;
  nodeType: BudgetNodeType;
  assignedAmount: string;
  goalStatus: "" | "active" | "completed" | "archived";
  spentAmount: string;
  remainingAmount: string;
};

type BudgetSignedDirection = "in" | "out" | "none";

type BudgetAmountParts = {
  number: string;
  prefix: string;
  text: string;
};

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

const props = defineProps<{
  period: string;
}>();

const assignedAmountMaxDigits = 8;
const assignedInputDisplaySize = 10;
const currency = "CLP";
const editingAssignedEnvelopeId = ref("");
const editingNameNodeId = ref("");
const isTreeEditing = ref(false);
const nextCategoryNumber = ref(3);
const nextSubcategoryNumber = ref(4);
const nextEnvelopeNumber = ref(5);
const selectedGoalId = ref("");
const goalForm = ref({
  targetAmount: "0",
  deadlineDate: "",
  status: "active" as "" | "active" | "completed" | "archived",
});
const { categories, subcategories, envelopes, goals } = useBudgetTree();
const expandedKeys = ref<TreeTableExpandedKeys>({
  "cat-living": true,
  "cat-planning": true,
  "sub-home": true,
  "sub-food": true,
  "sub-reserve": true,
  [INCOME_CATEGORY_ID]: true,
  [INCOME_SUBCATEGORY_ID]: true,
  [TRASH_CATEGORY_ID]: true,
  [TRASH_SUBCATEGORY_ID]: true,
});

const budgetTreeQuery = useQuery({
  key: () => ["budget-tree", props.period],
  query: () => getBudgetTree(props.period),
});

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

const budgetEnvelopes = computed<BudgetEnvelope[]>(() =>
  envelopes.value.map((envelope) => ({ ...envelope })),
);
const goalByEnvelopeId = computed(() =>
  Object.fromEntries(goals.value.map((goal) => [goal.envelopeId, goal])),
);
const activeBudgetEnvelopes = computed(() =>
  budgetEnvelopes.value.filter(
    (envelope) => goalByEnvelopeId.value[envelope.id]?.status !== "archived",
  ),
);
const assignableBudgetEnvelopes = computed(() =>
  activeBudgetEnvelopes.value.filter(
    (envelope) =>
      envelope.id !== INCOME_ENVELOPE_ID &&
      envelope.subcategoryId !== TRASH_SUBCATEGORY_ID,
  ),
);
const receivedIncomeAmount = computed(() =>
  parseAmount(
    budgetEnvelopes.value.find((envelope) => envelope.id === INCOME_ENVELOPE_ID)
      ?.spentAmount ?? "0",
  ),
);
const assignedBudgetAmount = computed(() =>
  assignableBudgetEnvelopes.value.reduce(
    (sum, envelope) => sum + parseAmount(envelope.assignedAmount),
    0,
  ),
);
const incomeLeftToAssign = computed(() =>
  formatDecimal(receivedIncomeAmount.value - assignedBudgetAmount.value),
);
const incomeLeftTone = computed(() =>
  parseAmount(incomeLeftToAssign.value) >= 0 ? "positive" : "negative",
);
const visibleEnvelopeCount = computed(
  () => activeBudgetEnvelopes.value.filter((envelope) => envelope.id !== INCOME_ENVELOPE_ID).length,
);
const selectedGoal = computed(() =>
  goals.value.find((goal) => goal.id === selectedGoalId.value),
);
const selectedGoalEnvelope = computed(() =>
  budgetEnvelopes.value.find(
    (envelope) => envelope.id === selectedGoal.value?.envelopeId,
  ),
);
const isGoalDrawerOpen = computed(() => selectedGoal.value !== undefined);
const suggestedGoalContribution = computed(() => {
  const goal = selectedGoal.value;
  const envelope = selectedGoalEnvelope.value;

  if (!goal || !envelope) {
    return "0";
  }

  return formatDecimal(
    suggestedContribution(
      goal.targetAmount,
      envelope.assignedAmount,
      goal.deadlineDate,
    ),
  );
});

const nodes = computed<TreeNode[]>(() =>
  categories.value.filter((category) => category.id !== INCOME_CATEGORY_ID).map((category) => {
    const categorySubcategories = subcategories.value.filter(
      (subcategory) => subcategory.categoryId === category.id,
    );
    const categoryChildren = categorySubcategories.map((subcategory) => {
      const subcategoryEnvelopes = activeBudgetEnvelopes.value.filter(
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
            goalStatus: goalByEnvelopeId.value[envelope.id]?.status ?? "",
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
  const savedTree = await saveBudgetTree(props.period, toBudgetTreeDto(props.period));
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
      activityAmount: "0",
      remainingAmount: "0",
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
  const archivedEnvelopeIds = goalEnvelopeIds(removedEnvelopeIds);
  categories.value = categories.value.filter(
    (category) => category.id !== categoryId,
  );
  subcategories.value = subcategories.value.filter(
    (subcategory) => subcategory.categoryId !== categoryId,
  );
  envelopes.value = envelopes.value.filter(
    (envelope) =>
      !removedSubcategoryIds.includes(envelope.subcategoryId) ||
      archivedEnvelopeIds.includes(envelope.id),
  );
  moveEnvelopesToTrash(archivedEnvelopeIds);
  archiveGoalsForEnvelopeIds(archivedEnvelopeIds);
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
  const archivedEnvelopeIds = goalEnvelopeIds(removedEnvelopeIds);
  subcategories.value = subcategories.value.filter(
    (subcategory) => subcategory.id !== subcategoryId,
  );
  envelopes.value = envelopes.value.filter(
    (envelope) =>
      envelope.subcategoryId !== subcategoryId ||
      archivedEnvelopeIds.includes(envelope.id),
  );
  moveEnvelopesToTrash(archivedEnvelopeIds);
  archiveGoalsForEnvelopeIds(archivedEnvelopeIds);
  clearRemovedNodeEditing([subcategoryId, ...removedEnvelopeIds]);
  void persistBudgetTree();
}

function deleteEnvelope(envelopeId: string) {
  if (!isTreeEditing.value || isTrashEnvelope(envelopes.value, envelopeId)) {
    return;
  }

  if (goalByEnvelopeId.value[envelopeId]) {
    moveEnvelopesToTrash([envelopeId]);
    archiveGoalsForEnvelopeIds([envelopeId]);
    clearRemovedNodeEditing([envelopeId]);
    void persistBudgetTree();
    return;
  }

  envelopes.value = envelopes.value.filter(
    (envelope) => envelope.id !== envelopeId,
  );
  clearRemovedNodeEditing([envelopeId]);
  void persistBudgetTree();
}

function moveEnvelopesToTrash(envelopeIds: string[]) {
  envelopes.value = envelopes.value.map((envelope) =>
    envelopeIds.includes(envelope.id)
      ? { ...envelope, subcategoryId: TRASH_SUBCATEGORY_ID }
      : envelope,
  );
}

function archiveGoalsForEnvelopeIds(envelopeIds: string[]) {
  goals.value = goals.value.map((goal) =>
    envelopeIds.includes(goal.envelopeId)
      ? { ...goal, status: "archived" }
      : goal,
  );
}

function goalEnvelopeIds(envelopeIds: string[]): string[] {
  return envelopeIds.filter((envelopeId) => goalByEnvelopeId.value[envelopeId]);
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

function isProtectedNode(node: { key: string }): boolean {
  return (
    node.key === INCOME_CATEGORY_ID ||
    node.key === INCOME_SUBCATEGORY_ID ||
    node.key === INCOME_ENVELOPE_ID ||
    isTrashNode(node)
  );
}

function canDeleteNode(node: BudgetTreeNode): boolean {
  if (isProtectedNode(node)) {
    return false;
  }

  return (
    node.data.nodeType !== "envelope" ||
    !isTrashEnvelope(envelopes.value, node.key)
  );
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
    goalStatus: "",
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
    goalStatus: "",
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

function updateEnvelopeAssignedAmountFromInput(
  envelopeId: string,
  assignedAmount: string,
) {
  updateEnvelopeAssignedAmount(
    envelopeId,
    assignedInputStorageValue(assignedAmount),
  );
}

function editAssignedAmount(envelopeId: string) {
  editingAssignedEnvelopeId.value = envelopeId;
}

function stopEditingAssignedAmount(envelopeId: string) {
  const envelope = envelopes.value.find((candidate) => candidate.id === envelopeId);

  if (envelope) {
    updateEnvelopeAssignedAmount(
      envelopeId,
      normalizedAmount(envelope.assignedAmount),
    );
  }

  if (editingAssignedEnvelopeId.value === envelopeId) {
    editingAssignedEnvelopeId.value = "";
  }

  void persistBudgetTree();
}

function openGoalDetails(envelopeId: string) {
  const goal = goals.value.find((candidate) => candidate.envelopeId === envelopeId);

  if (!goal) {
    return;
  }

  selectedGoalId.value = goal.id;
  goalForm.value = {
    targetAmount: goal.targetAmount,
    deadlineDate: goal.deadlineDate,
    status: goal.status,
  };
}

function closeGoalDetails() {
  selectedGoalId.value = "";
}

function saveGoalDetails() {
  const goalId = selectedGoalId.value;

  goals.value = goals.value.map((goal) =>
    goal.id === goalId
      ? {
          ...goal,
          targetAmount: normalizedAmount(goalForm.value.targetAmount),
          deadlineDate: goalForm.value.deadlineDate,
          status: goalForm.value.status || "active",
        }
      : goal,
  );
  closeGoalDetails();
  void persistBudgetTree();
}

function completeGoal() {
  goalForm.value = { ...goalForm.value, status: "completed" };
  saveGoalDetails();
}

function archiveGoal() {
  goalForm.value = { ...goalForm.value, status: "archived" };
  saveGoalDetails();
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

function assignedInputValue(amount: string): string {
  return formatAssignedInputValue(assignedInputStorageValue(amount));
}

function assignedInputStorageValue(amount: string): string {
  return amountDigits(amount).slice(0, assignedAmountMaxDigits);
}

function amountDigits(amount: string): string {
  return amount.replace(/\D/g, "");
}

function formatAssignedInputValue(amount: string): string {
  return amount.replace(/\B(?=(\d{3})+(?!\d))/g, ".");
}

function amountParts(amount: string): BudgetAmountParts {
  const formattedAmount = formatMoney(amount, currency);
  const amountMatch = /^(\D*)(.*)$/.exec(formattedAmount);

  if (!amountMatch) {
    return {
      number: formattedAmount,
      prefix: "",
      text: formattedAmount,
    };
  }

  return {
    number: amountMatch[2],
    prefix: amountMatch[1],
    text: formattedAmount,
  };
}

function signedSpentAmountParts(spentAmount: string): BudgetSignedAmountParts {
  const spentValue = parseAmount(spentAmount);
  const direction = spentDirection(spentValue);
  const sign = spentSign(direction);
  const parts = amountParts(formatDecimal(Math.abs(spentValue)));
  const prefix = `${sign}${parts.prefix}`;
  return {
    direction,
    number: parts.number,
    prefix,
    text: `${prefix}${parts.number}`,
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

function normalizedAmount(amount: string): string {
  return formatDecimal(Math.max(0, parseAmount(amount)));
}

function suggestedContribution(
  targetAmount: string,
  assignedAmount: string,
  deadlineDate: string,
): number {
  const remaining = Math.max(0, parseAmount(targetAmount) - parseAmount(assignedAmount));
  const months = monthsUntilDeadline(deadlineDate);
  return months <= 1 ? remaining : Math.ceil(remaining / months);
}

function monthsUntilDeadline(deadlineDate: string): number {
  if (deadlineDate === "") {
    return 1;
  }

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
          <BaseBadge>{{ visibleEnvelopeCount }} sobres</BaseBadge>
        </div>
      </div>
    </template>

    <section aria-labelledby="budget-heading">
      <div class="zero-budget-banner">
        <div>
          <p class="eyebrow">Presupuesto base cero</p>
          <strong>Ingresos por asignar</strong>
        </div>
        <span class="zero-budget-banner__amount" :data-tone="incomeLeftTone">
          {{ formatMoney(incomeLeftToAssign, currency) }}
        </span>
      </div>

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
                @click.stop="addCategory"
              >
                + Categoría
              </BaseButton>
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
                {{ node.data.name }}
                <BaseButton
                  v-if="node.data.goalStatus"
                  size="sm"
                  type="button"
                  variant="secondary"
                  :aria-label="`Abrir meta ${node.data.name}`"
                  @click="openGoalDetails(node.key)"
                >
                  Meta
                </BaseButton>
              </span>
              <span
                v-else-if="node.data.nodeType === 'subcategory'"
                class="group-label"
                data-level="subcategory"
                :data-subcategory-id="node.key"
              >
                {{ node.data.name }}
              </span>
              <span
                v-else
                class="group-label"
                data-level="category"
                :data-category-id="node.key"
              >
                {{ node.data.name }}
              </span>

              <BaseButton
                v-if="isTreeEditing && !isProtectedNode(node)"
                size="sm"
                type="button"
                variant="secondary"
                :aria-label="`Renombrar ${node.data.name}`"
                @click="editNodeName(node.key)"
              >
                Renombrar
              </BaseButton>
              <BaseButton
                v-if="
                  isTreeEditing &&
                  node.data.nodeType === 'category' &&
                  canDeleteNode(node)
                "
                size="sm"
                type="button"
                variant="secondary"
                :aria-label="`Agregar subcategoría en ${node.data.name}`"
                @click="addSubcategory(node.key)"
              >
                + Subcategoría
              </BaseButton>
              <BaseButton
                v-else-if="
                  isTreeEditing &&
                  node.data.nodeType === 'subcategory' &&
                  canDeleteNode(node)
                "
                size="sm"
                type="button"
                variant="secondary"
                :aria-label="`Agregar sobre en ${node.data.name}`"
                @click="addEnvelope(node.key)"
              >
                + Sobre
              </BaseButton>
              <BaseButton
                v-if="
                  isTreeEditing &&
                  node.data.nodeType === 'category' &&
                  !isProtectedNode(node)
                "
                size="sm"
                type="button"
                variant="danger"
                :aria-label="`Eliminar categoría ${node.data.name}`"
                @click="deleteCategory(node.key)"
              >
                Eliminar
              </BaseButton>
              <BaseButton
                v-else-if="
                  isTreeEditing &&
                  node.data.nodeType === 'subcategory' &&
                  !isProtectedNode(node)
                "
                size="sm"
                type="button"
                variant="danger"
                :aria-label="`Eliminar subcategoría ${node.data.name}`"
                @click="deleteSubcategory(node.key)"
              >
                Eliminar
              </BaseButton>
              <BaseButton
                v-else-if="
                  isTreeEditing &&
                  node.data.nodeType === 'envelope' &&
                  canDeleteNode(node)
                "
                size="sm"
                type="button"
                variant="danger"
                :aria-label="`Eliminar sobre ${node.data.name}`"
                @click="deleteEnvelope(node.key)"
              >
                Eliminar
              </BaseButton>
            </div>
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
            <BaseTextInput
              v-if="
                node.data.nodeType === 'envelope' &&
                editingAssignedEnvelopeId === node.key
              "
              v-assigned-input-focus
              :aria-label="`Asignado para ${node.data.name}`"
              class="assigned-input"
              inputmode="numeric"
              :maxlength="assignedInputDisplaySize"
              :model-value="assignedInputValue(node.data.assignedAmount)"
              pattern="[0-9.]*"
              :size="assignedInputDisplaySize"
              type="text"
              @update:model-value="
                (value) => updateEnvelopeAssignedAmountFromInput(node.key, value)
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
              <span class="numeric__prefix">
                {{ amountParts(node.data.assignedAmount).prefix }}
              </span>
              <span class="numeric__amount">
                {{ amountParts(node.data.assignedAmount).number }}
              </span>
            </button>
            <span
              v-else
              class="numeric"
              :aria-label="amountParts(node.data.assignedAmount).text"
            >
              <span class="numeric__prefix">
                {{ amountParts(node.data.assignedAmount).prefix }}
              </span>
              <span class="numeric__amount">
                {{ amountParts(node.data.assignedAmount).number }}
              </span>
            </span>
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
        </BaseColumn>

        <BaseColumn
          field="remainingAmount"
          header="Restante"
          body-class="amount-column-body"
          header-class="amount-column-header"
          style="width: 12rem"
        >
          <template #body="{ node }">
            <span
              class="numeric"
              :aria-label="amountParts(node.data.remainingAmount).text"
              :data-tone="amountTone(node.data.remainingAmount)"
            >
              <span class="numeric__prefix">
                {{ amountParts(node.data.remainingAmount).prefix }}
              </span>
              <span class="numeric__amount">
                {{ amountParts(node.data.remainingAmount).number }}
              </span>
            </span>
          </template>
        </BaseColumn>

        <template #empty>Sin sobres para mostrar.</template>
      </BaseTreeTable>
    </section>
  </BaseCard>

  <Teleport to="body">
    <div
      v-if="isGoalDrawerOpen"
      class="goal-drawer-backdrop"
      role="presentation"
      @click.self="closeGoalDetails"
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
              {{ selectedGoalEnvelope?.name ?? "Meta" }}
            </h2>
          </div>
          <BaseButton
            type="button"
            variant="secondary"
            @click="closeGoalDetails"
          >
            Cerrar
          </BaseButton>
        </header>

        <form class="goal-form" @submit.prevent="saveGoalDetails">
          <dl class="goal-summary">
            <div>
              <dt>Asignado este periodo</dt>
              <dd>
                {{
                  formatMoney(selectedGoalEnvelope?.assignedAmount ?? "0", currency)
                }}
              </dd>
            </div>
            <div>
              <dt>Sugerido mensual</dt>
              <dd>{{ formatMoney(suggestedGoalContribution, currency) }}</dd>
            </div>
            <div>
              <dt>Sobre vinculado</dt>
              <dd>{{ selectedGoalEnvelope?.name ?? "Sin sobre" }}</dd>
            </div>
          </dl>

          <BaseField input-id="goal-target" label="Monto objetivo">
            <BaseTextInput
              id="goal-target"
              v-model="goalForm.targetAmount"
              inputmode="decimal"
              pattern="\d+(\.\d+)?"
              required
            />
          </BaseField>

          <BaseField input-id="goal-deadline" label="Fecha objetivo">
            <BaseTextInput
              id="goal-deadline"
              v-model="goalForm.deadlineDate"
              type="date"
            />
          </BaseField>

          <div class="goal-form__actions">
            <BaseButton type="button" variant="secondary" @click="completeGoal">
              Completar
            </BaseButton>
            <BaseButton type="button" variant="danger" @click="archiveGoal">
              Archivar
            </BaseButton>
            <BaseButton type="submit">Guardar</BaseButton>
          </div>
        </form>
      </aside>
    </div>
  </Teleport>
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

.zero-budget-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--lu-space-4);
  border: 1px solid var(--lu-color-border);
  border-radius: var(--lu-radius-md);
  background: var(--lu-color-surface-muted);
  margin-block-end: var(--lu-space-4);
  padding: var(--lu-space-4);
}

.zero-budget-banner .eyebrow {
  margin: 0 0 var(--lu-space-1);
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-xs);
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.zero-budget-banner strong {
  color: var(--lu-color-text);
  font-size: var(--lu-font-size-md);
}

.zero-budget-banner__amount {
  color: var(--lu-color-text);
  font-size: var(--lu-font-size-2xl);
  font-weight: 900;
  white-space: nowrap;
}

.zero-budget-banner__amount[data-tone="negative"] {
  color: var(--lu-color-danger);
}

.budget-tree-table {
  font-size: var(--lu-font-size-md);
  font-weight: 400;
}

.budget-tree-table :deep(.p-treetable-thead > tr > th),
.budget-tree-table :deep(.p-treetable-tbody > tr > td) {
  font-size: inherit;
  font-weight: inherit;
}

.node-name-cell {
  display: inline-flex;
  align-items: center;
  gap: var(--lu-space-2);
  max-inline-size: 100%;
}

.name-column-header {
  display: inline-flex;
  align-items: center;
  gap: var(--lu-space-2);
}

.group-label,
.envelope-label {
  display: inline-flex;
  align-items: center;
  gap: var(--lu-space-2);
}

.group-label[data-level="subcategory"] {
  color: var(--lu-color-text-muted);
}

.node-name-input {
  min-inline-size: 10rem;
  max-inline-size: 16rem;
  min-block-size: var(--lu-size-control-sm);
}

.numeric {
  display: inline-flex;
  align-items: baseline;
  justify-content: flex-end;
  gap: var(--lu-space-1);
  color: var(--lu-color-text);
  inline-size: 100%;
  white-space: nowrap;
}

.numeric__amount {
  text-align: end;
}

.numeric[data-tone="negative"] {
  color: var(--lu-color-danger);
}

.numeric[data-direction="in"] {
  color: var(--lu-color-positive);
}

.numeric[data-direction="out"] {
  color: var(--lu-color-danger);
}

.assigned-display {
  border: 0;
  background: transparent;
  cursor: text;
  padding: 0;
}

.assigned-display:hover,
.assigned-display:focus-visible {
  color: var(--lu-color-primary);
  outline: 0;
}

.assigned-display:focus-visible {
  text-decoration: underline;
  text-underline-offset: 0.2em;
}

.assigned-input {
  box-sizing: content-box;
  inline-size: 10ch;
  max-inline-size: 10rem;
  min-block-size: var(--lu-size-control-sm);
  min-inline-size: 10ch;
  text-align: end;
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
