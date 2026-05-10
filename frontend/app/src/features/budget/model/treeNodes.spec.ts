import { describe, expect, it } from "vitest";

import {
  INCOME_CATEGORY_ID,
  type BudgetCategory,
  type BudgetEnvelope,
  type BudgetGoal,
  type BudgetSubcategory,
} from "@/features/budget/model/budgetTree";
import { buildBudgetTreeNodes } from "@/features/budget/model/treeNodes";

describe("budget tree nodes", () => {
  it("builds table nodes without the income category", () => {
    const categories: BudgetCategory[] = [
      { id: INCOME_CATEGORY_ID, name: "Ingresos" },
      { id: "cat-home", name: "Casa" },
    ];
    const subcategories: BudgetSubcategory[] = [
      { id: "sub-income", categoryId: INCOME_CATEGORY_ID, name: "Sueldo" },
      { id: "sub-home", categoryId: "cat-home", name: "Hogar" },
    ];
    const envelopes: BudgetEnvelope[] = [
      envelope("env-income", "sub-income", "Sueldo", "0", "1000000"),
      envelope("env-rent", "sub-home", "Arriendo", "300000", "250000"),
      envelope("env-food", "sub-home", "Comida", "200000", "50000"),
    ];
    const goals: Record<string, BudgetGoal> = {
      "env-food": {
        id: "goal-food",
        envelopeId: "env-food",
        targetAmount: "500000",
        deadlineDate: "2026-12-31",
        status: "active",
      },
    };

    const nodes = buildBudgetTreeNodes(
      categories,
      subcategories,
      envelopes,
      goals,
    );

    expect(nodes).toHaveLength(1);
    expect(nodes[0]?.key).toBe("cat-home");
    expect(nodes[0]?.data).toMatchObject({
      assignedAmount: "500000",
      remainingAmount: "200000",
      spentAmount: "300000",
    });
    expect(nodes[0]?.children?.[0]?.children?.[1]?.data).toMatchObject({
      goalStatus: "active",
      remainingAmount: "150000",
    });
  });
});

function envelope(
  id: string,
  subcategoryId: string,
  name: string,
  assignedAmount: string,
  spentAmount: string,
): BudgetEnvelope {
  return {
    id,
    subcategoryId,
    name,
    assignedAmount,
    activityAmount: spentAmount,
    remainingAmount: "0",
    spentAmount,
  };
}
