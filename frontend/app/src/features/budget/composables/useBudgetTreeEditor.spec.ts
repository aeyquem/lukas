import { computed, ref } from "vue";
import { describe, expect, it } from "vitest";

import {
  TRASH_SUBCATEGORY_ID,
  type BudgetCategory,
  type BudgetEnvelope,
  type BudgetGoal,
  type BudgetSubcategory,
} from "@/features/budget/model/budgetTree";
import { useBudgetTreeEditor } from "@/features/budget/composables/useBudgetTreeEditor";

describe("useBudgetTreeEditor", () => {
  it("moves goal-linked envelopes to trash when deleting a category", () => {
    const categories = ref<BudgetCategory[]>([
      { id: "cat-home", name: "Casa" },
      { id: "trash", name: "Trash" },
    ]);
    const subcategories = ref<BudgetSubcategory[]>([
      { id: "sub-home", categoryId: "cat-home", name: "Hogar" },
    ]);
    const envelopes = ref<BudgetEnvelope[]>([
      envelope("env-rent", "sub-home"),
      envelope("env-goal", "sub-home"),
    ]);
    const goals = ref<BudgetGoal[]>([
      {
        id: "goal-1",
        envelopeId: "env-goal",
        targetAmount: "500000",
        deadlineDate: "",
        status: "active",
      },
    ]);
    const expandedKeys = ref({});
    const goalByEnvelopeId = computed(() =>
      Object.fromEntries(goals.value.map((goal) => [goal.envelopeId, goal])),
    );
    const editor = useBudgetTreeEditor({
      categories,
      subcategories,
      envelopes,
      goals,
      goalByEnvelopeId,
      expandedKeys,
      persistBudgetTree: async () => undefined,
    });

    editor.toggleTreeEditing();
    editor.deleteCategory("cat-home");

    expect(categories.value.map((category) => category.id)).toEqual(["trash"]);
    expect(subcategories.value).toEqual([]);
    expect(envelopes.value).toEqual([
      {
        ...envelope("env-goal", "sub-home"),
        subcategoryId: TRASH_SUBCATEGORY_ID,
      },
    ]);
    expect(goals.value[0]?.status).toBe("archived");
  });
});

function envelope(id: string, subcategoryId: string): BudgetEnvelope {
  return {
    id,
    subcategoryId,
    name: id,
    assignedAmount: "0",
    activityAmount: "0",
    remainingAmount: "0",
    spentAmount: "0",
  };
}
