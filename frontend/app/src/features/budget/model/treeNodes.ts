import type { TreeNode } from "primevue/treenode";

import {
  INCOME_CATEGORY_ID,
  type BudgetCategory,
  type BudgetEnvelope,
  type BudgetGoal,
  type BudgetSubcategory,
} from "@/features/budget/model/budgetTree";
import { formatDecimal, parseAmount, remainingAmount } from "@/features/budget/model/amounts";

export type BudgetNodeType = "category" | "subcategory" | "envelope";

export type BudgetNodeData = {
  name: string;
  nodeType: BudgetNodeType;
  assignedAmount: string;
  goalStatus: "" | "active" | "completed" | "archived";
  spentAmount: string;
  remainingAmount: string;
};

export type BudgetTreeNode = TreeNode & {
  key: string;
  data: BudgetNodeData;
};

export function buildBudgetTreeNodes(
  categories: readonly BudgetCategory[],
  subcategories: readonly BudgetSubcategory[],
  envelopes: readonly BudgetEnvelope[],
  goalByEnvelopeId: Readonly<Record<string, BudgetGoal>>,
): TreeNode[] {
  return categories
    .filter((category) => category.id !== INCOME_CATEGORY_ID)
    .map((category) =>
      buildCategoryNode(category, subcategories, envelopes, goalByEnvelopeId),
    );
}

function buildCategoryNode(
  category: BudgetCategory,
  subcategories: readonly BudgetSubcategory[],
  envelopes: readonly BudgetEnvelope[],
  goalByEnvelopeId: Readonly<Record<string, BudgetGoal>>,
): BudgetTreeNode {
  const categorySubcategories = subcategories.filter(
    (subcategory) => subcategory.categoryId === category.id,
  );
  const children = categorySubcategories.map((subcategory) =>
    buildSubcategoryNode(subcategory, envelopes, goalByEnvelopeId),
  );

  return {
    key: category.id,
    data: buildParentNodeData(
      "category",
      category.name,
      children.map((child) => child.data),
    ),
    children,
  };
}

function buildSubcategoryNode(
  subcategory: BudgetSubcategory,
  envelopes: readonly BudgetEnvelope[],
  goalByEnvelopeId: Readonly<Record<string, BudgetGoal>>,
): BudgetTreeNode {
  const subcategoryEnvelopes = envelopes.filter(
    (envelope) => envelope.subcategoryId === subcategory.id,
  );

  return {
    key: subcategory.id,
    data: buildNodeData("subcategory", subcategory.name, subcategoryEnvelopes),
    children: subcategoryEnvelopes.map((envelope) =>
      buildEnvelopeNode(envelope, goalByEnvelopeId),
    ),
  };
}

function buildEnvelopeNode(
  envelope: BudgetEnvelope,
  goalByEnvelopeId: Readonly<Record<string, BudgetGoal>>,
): BudgetTreeNode {
  return {
    key: envelope.id,
    data: {
      name: envelope.name,
      nodeType: "envelope",
      assignedAmount: envelope.assignedAmount,
      goalStatus: goalByEnvelopeId[envelope.id]?.status ?? "",
      spentAmount: envelope.spentAmount,
      remainingAmount: remainingAmount(
        envelope.assignedAmount,
        envelope.spentAmount,
      ),
    },
  };
}

function buildNodeData(
  nodeType: BudgetNodeType,
  name: string,
  source: readonly BudgetEnvelope[],
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
  source: readonly BudgetNodeData[],
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

function sumEnvelopeField(
  source: readonly BudgetEnvelope[],
  field: "assignedAmount" | "spentAmount",
): string {
  const total = source.reduce(
    (sum, envelope) => sum + parseAmount(envelope[field]),
    0,
  );
  return formatDecimal(total);
}

function sumNodeField(
  source: readonly BudgetNodeData[],
  field: "assignedAmount" | "spentAmount",
): string {
  const total = source.reduce((sum, node) => sum + parseAmount(node[field]), 0);
  return formatDecimal(total);
}
