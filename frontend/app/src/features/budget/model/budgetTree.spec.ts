import { describe, expect, it } from "vitest";

import {
  TRASH_SUBCATEGORY_ID,
  isTrashEnvelope,
  type BudgetEnvelope,
} from "@/features/budget/model/budgetTree";

describe("budget tree", () => {
  it("identifies envelopes that are already in trash", () => {
    const envelopes: BudgetEnvelope[] = [
      {
        id: "active-envelope",
        subcategoryId: "active-subcategory",
        name: "Casa",
        assignedAmount: "0",
        activityAmount: "0",
        remainingAmount: "0",
        spentAmount: "0",
      },
      {
        id: "trashed-envelope",
        subcategoryId: TRASH_SUBCATEGORY_ID,
        name: "Arriendo",
        assignedAmount: "0",
        activityAmount: "0",
        remainingAmount: "0",
        spentAmount: "0",
      },
    ];

    expect(isTrashEnvelope(envelopes, "trashed-envelope")).toBe(true);
    expect(isTrashEnvelope(envelopes, "active-envelope")).toBe(false);
    expect(isTrashEnvelope(envelopes, "missing-envelope")).toBe(false);
  });
});
