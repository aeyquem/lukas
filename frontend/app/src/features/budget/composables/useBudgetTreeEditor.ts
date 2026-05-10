import { ref, type ComputedRef, type Ref } from "vue";
import type { TreeTableExpandedKeys } from "primevue/treetable";

import {
  INCOME_CATEGORY_ID,
  INCOME_ENVELOPE_ID,
  INCOME_SUBCATEGORY_ID,
  TRASH_CATEGORY_ID,
  TRASH_SUBCATEGORY_ID,
  isTrashEnvelope,
  type BudgetCategory,
  type BudgetEnvelope,
  type BudgetGoal,
  type BudgetSubcategory,
} from "@/features/budget/model/budgetTree";
import { assignedInputStorageValue, normalizedAmount } from "@/features/budget/model/amounts";
import type { BudgetNodeType, BudgetTreeNode } from "@/features/budget/model/treeNodes";

type BudgetTreeEditorOptions = {
  categories: Ref<BudgetCategory[]>;
  subcategories: Ref<BudgetSubcategory[]>;
  envelopes: Ref<BudgetEnvelope[]>;
  goals: Ref<BudgetGoal[]>;
  goalByEnvelopeId: ComputedRef<Record<string, BudgetGoal>>;
  expandedKeys: Ref<TreeTableExpandedKeys>;
  persistBudgetTree: () => Promise<void>;
};

type BudgetTreeEditorContext = BudgetTreeEditorOptions & {
  editingAssignedEnvelopeId: Ref<string>;
  editingNameNodeId: Ref<string>;
  isTreeEditing: Ref<boolean>;
  nextCategoryNumber: Ref<number>;
  nextEnvelopeNumber: Ref<number>;
  nextSubcategoryNumber: Ref<number>;
};

type DropEdge = "before" | "after";

export type EnvelopeDropInstruction = {
  targetEnvelopeId: string;
  edge: DropEdge;
};

export type SubcategoryDropInstruction = {
  targetSubcategoryId: string;
  edge: DropEdge;
};

export type CategoryDropInstruction = {
  targetCategoryId: string;
  edge: DropEdge;
};

export function useBudgetTreeEditor(options: BudgetTreeEditorOptions) {
  const context = createContext(options);

  return {
    editingAssignedEnvelopeId: context.editingAssignedEnvelopeId,
    editingNameNodeId: context.editingNameNodeId,
    isTreeEditing: context.isTreeEditing,
    addCategory: () => addCategory(context),
    addEnvelope: (id: string) => addEnvelope(context, id),
    addSubcategory: (id: string) => addSubcategory(context, id),
    canDeleteNode: (node: BudgetTreeNode) => canDeleteNode(context, node),
    completeEditingNodeName: (type: BudgetNodeType, id: string) =>
      completeEditingNodeName(context, type, id),
    deleteCategory: (id: string) => deleteCategory(context, id),
    deleteEnvelope: (id: string) => deleteEnvelope(context, id),
    deleteSubcategory: (id: string) => deleteSubcategory(context, id),
    editAssignedAmount: (id: string) => editAssignedAmount(context, id),
    editNodeName: (id: string) => editNodeName(context, id),
    expandExistingBudgetTree: () => expandExistingBudgetTree(context),
    isProtectedNode,
    moveCategoryNear: (id: string, instruction: CategoryDropInstruction) =>
      moveCategoryNear(context, id, instruction),
    moveEnvelope: (id: string, targetId: string) =>
      moveEnvelope(context, id, targetId),
    moveEnvelopeNear: (id: string, instruction: EnvelopeDropInstruction) =>
      moveEnvelopeNear(context, id, instruction),
    moveSubcategory: (id: string, targetId: string) =>
      moveSubcategory(context, id, targetId),
    moveSubcategoryNear: (
      id: string,
      instruction: SubcategoryDropInstruction,
    ) => moveSubcategoryNear(context, id, instruction),
    stopEditingAssignedAmount: (id: string) =>
      stopEditingAssignedAmount(context, id),
    toggleTreeEditing: () => toggleTreeEditing(context),
    updateEnvelopeAssignedAmountFromInput: (id: string, amount: string) =>
      updateEnvelopeAssignedAmountFromInput(context, id, amount),
    updateNodeName: (type: BudgetNodeType, id: string, name: string) =>
      updateNodeName(context, type, id, name),
  };
}

function createContext(
  options: BudgetTreeEditorOptions,
): BudgetTreeEditorContext {
  return {
    ...options,
    editingAssignedEnvelopeId: ref(""),
    editingNameNodeId: ref(""),
    isTreeEditing: ref(false),
    nextCategoryNumber: ref(3),
    nextEnvelopeNumber: ref(5),
    nextSubcategoryNumber: ref(4),
  };
}

function toggleTreeEditing(context: BudgetTreeEditorContext) {
  context.isTreeEditing.value = !context.isTreeEditing.value;

  if (!context.isTreeEditing.value) {
    context.editingNameNodeId.value = "";
  }
}

function addCategory(context: BudgetTreeEditorContext) {
  if (!context.isTreeEditing.value) {
    return;
  }

  const categoryNumber = context.nextCategoryNumber.value;
  const categoryId = crypto.randomUUID();
  context.nextCategoryNumber.value = categoryNumber + 1;
  context.categories.value = [
    ...context.categories.value,
    { id: categoryId, name: `Nueva categoría ${categoryNumber}` },
  ];
  expandCategory(context, categoryId);
  context.editingNameNodeId.value = categoryId;
  void context.persistBudgetTree();
}

function addSubcategory(context: BudgetTreeEditorContext, categoryId: string) {
  if (!context.isTreeEditing.value) {
    return;
  }

  const subcategoryNumber = context.nextSubcategoryNumber.value;
  const subcategoryId = crypto.randomUUID();
  context.nextSubcategoryNumber.value = subcategoryNumber + 1;
  context.subcategories.value = [
    ...context.subcategories.value,
    {
      id: subcategoryId,
      categoryId,
      name: `Nueva subcategoría ${subcategoryNumber}`,
    },
  ];
  expandSubcategoryParent(context, categoryId, subcategoryId);
  context.editingNameNodeId.value = subcategoryId;
  void context.persistBudgetTree();
}

function addEnvelope(context: BudgetTreeEditorContext, subcategoryId: string) {
  if (!context.isTreeEditing.value) {
    return;
  }

  const envelopeNumber = context.nextEnvelopeNumber.value;
  const envelopeId = crypto.randomUUID();
  context.nextEnvelopeNumber.value = envelopeNumber + 1;
  context.envelopes.value = [
    ...context.envelopes.value,
    newEnvelope(envelopeId, subcategoryId, `Nuevo sobre ${envelopeNumber}`),
  ];
  expandEnvelopeParent(context, subcategoryId);
  context.editingNameNodeId.value = envelopeId;
  void context.persistBudgetTree();
}

function deleteCategory(context: BudgetTreeEditorContext, categoryId: string) {
  if (!context.isTreeEditing.value) {
    return;
  }

  const removedSubcategoryIds = context.subcategories.value
    .filter((subcategory) => subcategory.categoryId === categoryId)
    .map((subcategory) => subcategory.id);
  const removedEnvelopeIds = context.envelopes.value
    .filter((envelope) =>
      removedSubcategoryIds.includes(envelope.subcategoryId),
    )
    .map((envelope) => envelope.id);
  const archivedEnvelopeIds = goalEnvelopeIds(context, removedEnvelopeIds);
  context.categories.value = context.categories.value.filter(
    (category) => category.id !== categoryId,
  );
  context.subcategories.value = context.subcategories.value.filter(
    (subcategory) => subcategory.categoryId !== categoryId,
  );
  removeOrTrashEnvelopes(context, removedSubcategoryIds, archivedEnvelopeIds);
  archiveGoalsForEnvelopeIds(context, archivedEnvelopeIds);
  clearRemovedNodeEditing(context, [
    categoryId,
    ...removedSubcategoryIds,
    ...removedEnvelopeIds,
  ]);
  void context.persistBudgetTree();
}

function deleteSubcategory(
  context: BudgetTreeEditorContext,
  subcategoryId: string,
) {
  if (!context.isTreeEditing.value) {
    return;
  }

  const removedEnvelopeIds = context.envelopes.value
    .filter((envelope) => envelope.subcategoryId === subcategoryId)
    .map((envelope) => envelope.id);
  const archivedEnvelopeIds = goalEnvelopeIds(context, removedEnvelopeIds);
  context.subcategories.value = context.subcategories.value.filter(
    (subcategory) => subcategory.id !== subcategoryId,
  );
  context.envelopes.value = context.envelopes.value.filter(
    (envelope) =>
      envelope.subcategoryId !== subcategoryId ||
      archivedEnvelopeIds.includes(envelope.id),
  );
  moveEnvelopesToTrash(context, archivedEnvelopeIds);
  archiveGoalsForEnvelopeIds(context, archivedEnvelopeIds);
  clearRemovedNodeEditing(context, [subcategoryId, ...removedEnvelopeIds]);
  void context.persistBudgetTree();
}

function deleteEnvelope(context: BudgetTreeEditorContext, envelopeId: string) {
  if (
    !context.isTreeEditing.value ||
    isTrashEnvelope(context.envelopes.value, envelopeId)
  ) {
    return;
  }

  if (context.goalByEnvelopeId.value[envelopeId]) {
    moveEnvelopesToTrash(context, [envelopeId]);
    archiveGoalsForEnvelopeIds(context, [envelopeId]);
    clearRemovedNodeEditing(context, [envelopeId]);
    void context.persistBudgetTree();
    return;
  }

  context.envelopes.value = context.envelopes.value.filter(
    (envelope) => envelope.id !== envelopeId,
  );
  clearRemovedNodeEditing(context, [envelopeId]);
  void context.persistBudgetTree();
}

function editNodeName(context: BudgetTreeEditorContext, nodeId: string) {
  if (!context.isTreeEditing.value) {
    return;
  }

  context.editingNameNodeId.value = nodeId;
}

function updateNodeName(
  context: BudgetTreeEditorContext,
  nodeType: BudgetNodeType,
  nodeId: string,
  name: string,
) {
  if (nodeType === "category") {
    updateCategoryName(context, nodeId, name);
    return;
  }

  if (nodeType === "subcategory") {
    updateSubcategoryName(context, nodeId, name);
    return;
  }

  updateEnvelopeName(context, nodeId, name);
}

function completeEditingNodeName(
  context: BudgetTreeEditorContext,
  nodeType: BudgetNodeType,
  nodeId: string,
) {
  const nodeName = nodeNameById(context, nodeType, nodeId).trim();
  const fallbackName = defaultNodeName(nodeType);
  updateNodeName(context, nodeType, nodeId, nodeName || fallbackName);

  if (context.editingNameNodeId.value === nodeId) {
    context.editingNameNodeId.value = "";
  }

  void context.persistBudgetTree();
}

function updateEnvelopeAssignedAmountFromInput(
  context: BudgetTreeEditorContext,
  envelopeId: string,
  assignedAmount: string,
) {
  updateEnvelopeAssignedAmount(
    context,
    envelopeId,
    assignedInputStorageValue(assignedAmount),
  );
}

function editAssignedAmount(
  context: BudgetTreeEditorContext,
  envelopeId: string,
) {
  context.editingAssignedEnvelopeId.value = envelopeId;
}

function stopEditingAssignedAmount(
  context: BudgetTreeEditorContext,
  envelopeId: string,
) {
  const envelope = context.envelopes.value.find(
    (candidate) => candidate.id === envelopeId,
  );

  if (envelope) {
    updateEnvelopeAssignedAmount(
      context,
      envelopeId,
      normalizedAmount(envelope.assignedAmount),
    );
  }

  if (context.editingAssignedEnvelopeId.value === envelopeId) {
    context.editingAssignedEnvelopeId.value = "";
  }

  void context.persistBudgetTree();
}

function moveEnvelope(
  context: BudgetTreeEditorContext,
  envelopeId: string,
  targetSubcategoryId: string,
) {
  const targetSubcategory = context.subcategories.value.find(
    (subcategory) => subcategory.id === targetSubcategoryId,
  );

  if (!targetSubcategory) {
    return;
  }

  context.envelopes.value = context.envelopes.value.map((envelope) =>
    envelope.id === envelopeId
      ? { ...envelope, subcategoryId: targetSubcategoryId }
      : envelope,
  );
  expandEnvelopeParent(context, targetSubcategory.id);
  void context.persistBudgetTree();
}

function moveEnvelopeNear(
  context: BudgetTreeEditorContext,
  envelopeId: string,
  instruction: EnvelopeDropInstruction,
) {
  const draggedEnvelope = context.envelopes.value.find(
    (envelope) => envelope.id === envelopeId,
  );
  const targetEnvelope = context.envelopes.value.find(
    (envelope) => envelope.id === instruction.targetEnvelopeId,
  );

  if (!draggedEnvelope || !targetEnvelope) {
    return;
  }

  insertEnvelopeNear(
    context,
    draggedEnvelope,
    targetEnvelope,
    instruction.edge,
  );
  expandEnvelopeParent(context, targetEnvelope.subcategoryId);
  void context.persistBudgetTree();
}

function moveSubcategory(
  context: BudgetTreeEditorContext,
  subcategoryId: string,
  targetCategoryId: string,
) {
  const draggedSubcategory = context.subcategories.value.find(
    (subcategory) => subcategory.id === subcategoryId,
  );
  const targetCategory = context.categories.value.find(
    (category) => category.id === targetCategoryId,
  );

  if (!draggedSubcategory || !targetCategory) {
    return;
  }

  context.subcategories.value = [
    ...context.subcategories.value.filter(
      (subcategory) => subcategory.id !== subcategoryId,
    ),
    { ...draggedSubcategory, categoryId: targetCategory.id },
  ];
  expandSubcategoryParent(context, targetCategory.id, draggedSubcategory.id);
  void context.persistBudgetTree();
}

function moveSubcategoryNear(
  context: BudgetTreeEditorContext,
  subcategoryId: string,
  instruction: SubcategoryDropInstruction,
) {
  const draggedSubcategory = context.subcategories.value.find(
    (subcategory) => subcategory.id === subcategoryId,
  );
  const targetSubcategory = context.subcategories.value.find(
    (subcategory) => subcategory.id === instruction.targetSubcategoryId,
  );

  if (!draggedSubcategory || !targetSubcategory) {
    return;
  }

  insertSubcategoryNear(
    context,
    draggedSubcategory,
    targetSubcategory,
    instruction.edge,
  );
  expandSubcategoryParent(context, targetSubcategory.categoryId, subcategoryId);
  void context.persistBudgetTree();
}

function moveCategoryNear(
  context: BudgetTreeEditorContext,
  categoryId: string,
  instruction: CategoryDropInstruction,
) {
  const draggedCategory = context.categories.value.find(
    (category) => category.id === categoryId,
  );

  if (!draggedCategory) {
    return;
  }

  const inserted = insertCategoryNear(
    context,
    draggedCategory,
    instruction.targetCategoryId,
    instruction.edge,
  );

  if (inserted) {
    expandCategory(context, draggedCategory.id);
    void context.persistBudgetTree();
  }
}

function canDeleteNode(
  context: BudgetTreeEditorContext,
  node: BudgetTreeNode,
): boolean {
  if (isProtectedNode(node)) {
    return false;
  }

  return (
    node.data.nodeType !== "envelope" ||
    !isTrashEnvelope(context.envelopes.value, node.key)
  );
}

function expandExistingBudgetTree(context: BudgetTreeEditorContext) {
  context.expandedKeys.value = {
    ...context.expandedKeys.value,
    ...Object.fromEntries(
      context.categories.value.map((category) => [category.id, true]),
    ),
    ...Object.fromEntries(
      context.subcategories.value.map((subcategory) => [subcategory.id, true]),
    ),
  };
}

function isProtectedNode(node: { key: string }): boolean {
  return (
    node.key === INCOME_CATEGORY_ID ||
    node.key === INCOME_SUBCATEGORY_ID ||
    node.key === INCOME_ENVELOPE_ID ||
    node.key === TRASH_CATEGORY_ID ||
    node.key === TRASH_SUBCATEGORY_ID
  );
}

function removeOrTrashEnvelopes(
  context: BudgetTreeEditorContext,
  removedSubcategoryIds: string[],
  archivedEnvelopeIds: string[],
) {
  context.envelopes.value = context.envelopes.value.filter(
    (envelope) =>
      !removedSubcategoryIds.includes(envelope.subcategoryId) ||
      archivedEnvelopeIds.includes(envelope.id),
  );
  moveEnvelopesToTrash(context, archivedEnvelopeIds);
}

function moveEnvelopesToTrash(
  context: BudgetTreeEditorContext,
  envelopeIds: string[],
) {
  context.envelopes.value = context.envelopes.value.map((envelope) =>
    envelopeIds.includes(envelope.id)
      ? { ...envelope, subcategoryId: TRASH_SUBCATEGORY_ID }
      : envelope,
  );
}

function archiveGoalsForEnvelopeIds(
  context: BudgetTreeEditorContext,
  envelopeIds: string[],
) {
  context.goals.value = context.goals.value.map((goal) =>
    envelopeIds.includes(goal.envelopeId)
      ? { ...goal, status: "archived" }
      : goal,
  );
}

function goalEnvelopeIds(
  context: BudgetTreeEditorContext,
  envelopeIds: string[],
): string[] {
  return envelopeIds.filter(
    (envelopeId) => context.goalByEnvelopeId.value[envelopeId],
  );
}

function clearRemovedNodeEditing(
  context: BudgetTreeEditorContext,
  nodeIds: string[],
) {
  if (nodeIds.includes(context.editingNameNodeId.value)) {
    context.editingNameNodeId.value = "";
  }

  if (nodeIds.includes(context.editingAssignedEnvelopeId.value)) {
    context.editingAssignedEnvelopeId.value = "";
  }
}

function updateCategoryName(
  context: BudgetTreeEditorContext,
  nodeId: string,
  name: string,
) {
  context.categories.value = context.categories.value.map((category) =>
    category.id === nodeId ? { ...category, name } : category,
  );
}

function updateSubcategoryName(
  context: BudgetTreeEditorContext,
  nodeId: string,
  name: string,
) {
  context.subcategories.value = context.subcategories.value.map(
    (subcategory) =>
      subcategory.id === nodeId ? { ...subcategory, name } : subcategory,
  );
}

function updateEnvelopeName(
  context: BudgetTreeEditorContext,
  nodeId: string,
  name: string,
) {
  context.envelopes.value = context.envelopes.value.map((envelope) =>
    envelope.id === nodeId ? { ...envelope, name } : envelope,
  );
}

function nodeNameById(
  context: BudgetTreeEditorContext,
  nodeType: BudgetNodeType,
  nodeId: string,
): string {
  if (nodeType === "category") {
    return (
      context.categories.value.find((category) => category.id === nodeId)
        ?.name ?? ""
    );
  }

  if (nodeType === "subcategory") {
    return (
      context.subcategories.value.find(
        (subcategory) => subcategory.id === nodeId,
      )?.name ?? ""
    );
  }

  return (
    context.envelopes.value.find((envelope) => envelope.id === nodeId)?.name ??
    ""
  );
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

function updateEnvelopeAssignedAmount(
  context: BudgetTreeEditorContext,
  envelopeId: string,
  assignedAmount: string,
) {
  context.envelopes.value = context.envelopes.value.map((envelope) =>
    envelope.id === envelopeId ? { ...envelope, assignedAmount } : envelope,
  );
}

function insertEnvelopeNear(
  context: BudgetTreeEditorContext,
  draggedEnvelope: BudgetEnvelope,
  targetEnvelope: BudgetEnvelope,
  edge: DropEdge,
) {
  const withoutDraggedEnvelope = context.envelopes.value.filter(
    (envelope) => envelope.id !== draggedEnvelope.id,
  );
  const targetIndex = withoutDraggedEnvelope.findIndex(
    (envelope) => envelope.id === targetEnvelope.id,
  );

  if (targetIndex < 0) {
    return;
  }

  const insertionIndex = edge === "before" ? targetIndex : targetIndex + 1;
  context.envelopes.value = [
    ...withoutDraggedEnvelope.slice(0, insertionIndex),
    { ...draggedEnvelope, subcategoryId: targetEnvelope.subcategoryId },
    ...withoutDraggedEnvelope.slice(insertionIndex),
  ];
}

function insertSubcategoryNear(
  context: BudgetTreeEditorContext,
  draggedSubcategory: BudgetSubcategory,
  targetSubcategory: BudgetSubcategory,
  edge: DropEdge,
) {
  const withoutDraggedSubcategory = context.subcategories.value.filter(
    (subcategory) => subcategory.id !== draggedSubcategory.id,
  );
  const targetIndex = withoutDraggedSubcategory.findIndex(
    (subcategory) => subcategory.id === targetSubcategory.id,
  );

  if (targetIndex < 0) {
    return;
  }

  const insertionIndex = edge === "before" ? targetIndex : targetIndex + 1;
  context.subcategories.value = [
    ...withoutDraggedSubcategory.slice(0, insertionIndex),
    { ...draggedSubcategory, categoryId: targetSubcategory.categoryId },
    ...withoutDraggedSubcategory.slice(insertionIndex),
  ];
}

function insertCategoryNear(
  context: BudgetTreeEditorContext,
  draggedCategory: BudgetCategory,
  targetCategoryId: string,
  edge: DropEdge,
): boolean {
  const withoutDraggedCategory = context.categories.value.filter(
    (category) => category.id !== draggedCategory.id,
  );
  const targetIndex = withoutDraggedCategory.findIndex(
    (category) => category.id === targetCategoryId,
  );

  if (targetIndex < 0) {
    return false;
  }

  const insertionIndex = edge === "before" ? targetIndex : targetIndex + 1;
  context.categories.value = [
    ...withoutDraggedCategory.slice(0, insertionIndex),
    draggedCategory,
    ...withoutDraggedCategory.slice(insertionIndex),
  ];
  return true;
}

function expandEnvelopeParent(
  context: BudgetTreeEditorContext,
  subcategoryId: string,
) {
  const subcategory = context.subcategories.value.find(
    (candidate) => candidate.id === subcategoryId,
  );

  if (!subcategory) {
    return;
  }

  context.expandedKeys.value = {
    ...context.expandedKeys.value,
    [subcategory.categoryId]: true,
    [subcategory.id]: true,
  };
}

function expandSubcategoryParent(
  context: BudgetTreeEditorContext,
  categoryId: string,
  subcategoryId: string,
) {
  context.expandedKeys.value = {
    ...context.expandedKeys.value,
    [categoryId]: true,
    [subcategoryId]: true,
  };
}

function expandCategory(context: BudgetTreeEditorContext, categoryId: string) {
  context.expandedKeys.value = {
    ...context.expandedKeys.value,
    [categoryId]: true,
  };
}

function newEnvelope(
  id: string,
  subcategoryId: string,
  name: string,
): BudgetEnvelope {
  return {
    id,
    subcategoryId,
    name,
    assignedAmount: "0",
    activityAmount: "0",
    remainingAmount: "0",
    spentAmount: "0",
  };
}
