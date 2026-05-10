import type { VNode } from "vue";
import {
  draggable,
  dropTargetForElements,
} from "@atlaskit/pragmatic-drag-and-drop/element/adapter";
import type {
  TreeTablePassThroughMethodOptions,
  TreeTablePassThroughOptions,
} from "primevue/treetable";

import type { BudgetTreeNode } from "@/features/budget/model/treeNodes";
import type {
  CategoryDropInstruction,
  EnvelopeDropInstruction,
  SubcategoryDropInstruction,
} from "@/features/budget/composables/useBudgetTreeEditor";

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

type BudgetDragDropOptions = {
  moveCategoryNear: (
    categoryId: string,
    instruction: CategoryDropInstruction,
  ) => void;
  moveEnvelope: (envelopeId: string, targetSubcategoryId: string) => void;
  moveEnvelopeNear: (
    envelopeId: string,
    instruction: EnvelopeDropInstruction,
  ) => void;
  moveSubcategory: (subcategoryId: string, targetCategoryId: string) => void;
  moveSubcategoryNear: (
    subcategoryId: string,
    instruction: SubcategoryDropInstruction,
  ) => void;
};

export function useBudgetDragDrop(options: BudgetDragDropOptions) {
  const treeTablePassThrough: TreeTablePassThroughOptions = {
    row(rowOptions) {
      const node = (rowOptions as BudgetRowPassThroughOptions).instance?.node;
      return node ? buildRowPassThrough(options, node) : undefined;
    },
  };

  return {
    treeTablePassThrough,
  };
}

function buildRowPassThrough(
  options: BudgetDragDropOptions,
  node: BudgetTreeNode,
) {
  return {
    class: ["budget-row", `budget-row--${node.data.nodeType}`],
    draggable: "true",
    "data-budget-node-id": node.key,
    "data-budget-node-type": node.data.nodeType,
    onVnodeMounted(vnode: VNode) {
      registerBudgetRow(options, getVnodeElement(vnode), node);
    },
    onVnodeUpdated(vnode: VNode) {
      registerBudgetRow(options, getVnodeElement(vnode), node);
    },
    onVnodeBeforeUnmount(vnode: VNode) {
      cleanupRowRegistration(getVnodeElement(vnode));
    },
  };
}

function registerBudgetRow(
  options: BudgetDragDropOptions,
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
  registerNodeDragAndDrop(options, element, node);
}

function registerNodeDragAndDrop(
  options: BudgetDragDropOptions,
  element: DragElement,
  node: BudgetTreeNode,
) {
  if (node.data.nodeType === "category") {
    registerDraggableCategory(element, node.key);
    registerCategoryDropTarget(options, element, node.key);
    return;
  }

  if (node.data.nodeType === "subcategory") {
    registerDraggableSubcategory(element, node.key);
    registerSubcategoryDropTarget(options, element, node.key);
    return;
  }

  registerDraggableEnvelope(element, node.key);
  registerEnvelopeDropTarget(options, element, node.key);
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

function registerEnvelopeDropTarget(
  options: BudgetDragDropOptions,
  element: DragElement,
  envelopeId: string,
) {
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
      return envelopePlacementData(element, envelopeId, input.clientY);
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
      dropEnvelopeNear(options, source.data.envelopeId, self.data, element);
    },
  });
}

function registerSubcategoryDropTarget(
  options: BudgetDragDropOptions,
  element: DragElement,
  subcategoryId: string,
) {
  cleanupDropTargetRegistration(element);
  element.budgetDropTargetKey = subcategoryId;
  element.budgetDropTargetCleanup = dropTargetForElements({
    element,
    canDrop({ source }) {
      return canDropOnSubcategory(source.data, subcategoryId);
    },
    getData({ input }) {
      return subcategoryPlacementData(element, subcategoryId, input.clientY);
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
      dropOnSubcategory(
        options,
        source.data,
        self.data,
        subcategoryId,
        element,
      );
    },
  });
}

function registerCategoryDropTarget(
  options: BudgetDragDropOptions,
  element: DragElement,
  categoryId: string,
) {
  cleanupDropTargetRegistration(element);
  element.budgetDropTargetKey = categoryId;
  element.budgetDropTargetCleanup = dropTargetForElements({
    element,
    canDrop({ source }) {
      return canDropOnCategory(source.data, categoryId);
    },
    getData({ input }) {
      return categoryPlacementData(element, categoryId, input.clientY);
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
      dropOnCategory(options, source.data, self.data, categoryId, element);
    },
  });
}

function dropEnvelopeNear(
  options: BudgetDragDropOptions,
  envelopeId: unknown,
  data: Record<string | symbol, unknown>,
  element: HTMLElement,
) {
  delete element.dataset.dragEdge;
  if (typeof envelopeId !== "string") {
    return;
  }

  const instruction = parseEnvelopeDropInstruction(data);
  if (instruction) {
    options.moveEnvelopeNear(envelopeId, instruction);
  }
}

function dropOnSubcategory(
  options: BudgetDragDropOptions,
  source: Record<string | symbol, unknown>,
  data: Record<string | symbol, unknown>,
  subcategoryId: string,
  element: HTMLElement,
) {
  clearDropState(element);
  if (typeof source.envelopeId === "string") {
    options.moveEnvelope(source.envelopeId, subcategoryId);
    return;
  }

  if (typeof source.subcategoryId !== "string") {
    return;
  }

  const instruction = parseSubcategoryDropInstruction(data);
  if (instruction) {
    options.moveSubcategoryNear(source.subcategoryId, instruction);
  }
}

function dropOnCategory(
  options: BudgetDragDropOptions,
  source: Record<string | symbol, unknown>,
  data: Record<string | symbol, unknown>,
  categoryId: string,
  element: HTMLElement,
) {
  clearDropState(element);
  if (typeof source.subcategoryId === "string") {
    options.moveSubcategory(source.subcategoryId, categoryId);
    return;
  }

  if (typeof source.categoryId !== "string") {
    return;
  }

  const instruction = parseCategoryDropInstruction(data);
  if (instruction) {
    options.moveCategoryNear(source.categoryId, instruction);
  }
}

function getVnodeElement(vnode: VNode): DragElement | undefined {
  return vnode.el instanceof HTMLElement ? vnode.el : undefined;
}

function canStartRowDrag(clientX: number, clientY: number): boolean {
  const target = document.elementFromPoint(clientX, clientY);
  return !target?.closest(
    "button, input, textarea, select, a, [contenteditable]",
  );
}

function canDropOnSubcategory(
  source: Record<string | symbol, unknown>,
  subcategoryId: string,
): boolean {
  return (
    source.type === "budget-envelope" ||
    (source.type === "budget-subcategory" &&
      source.subcategoryId !== subcategoryId)
  );
}

function canDropOnCategory(
  source: Record<string | symbol, unknown>,
  categoryId: string,
): boolean {
  return (
    source.type === "budget-subcategory" ||
    (source.type === "budget-category" && source.categoryId !== categoryId)
  );
}

function envelopePlacementData(
  element: HTMLElement,
  envelopeId: string,
  clientY: number,
) {
  return {
    type: "budget-envelope-placement",
    targetEnvelopeId: envelopeId,
    edge: dropEdge(element, clientY),
  };
}

function subcategoryPlacementData(
  element: HTMLElement,
  subcategoryId: string,
  clientY: number,
) {
  return {
    type: "budget-subcategory",
    subcategoryId,
    targetSubcategoryId: subcategoryId,
    edge: dropEdge(element, clientY),
  };
}

function categoryPlacementData(
  element: HTMLElement,
  categoryId: string,
  clientY: number,
) {
  return {
    type: "budget-category",
    categoryId,
    targetCategoryId: categoryId,
    edge: dropEdge(element, clientY),
  };
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
