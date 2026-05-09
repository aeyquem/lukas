import { computed, ref } from "vue";

import type { BudgetGoalStatus, BudgetTreeDto } from "@/api/ledger";

export const TRASH_CATEGORY_ID = "018f9df8-6c1f-7c1a-9a32-700000000103";
export const TRASH_SUBCATEGORY_ID =
  "018f9df8-6c1f-7c1a-9a32-700000000203";
export const INCOME_CATEGORY_ID = "018f9df8-6c1f-7c1a-9a32-700000000104";
export const INCOME_SUBCATEGORY_ID =
  "018f9df8-6c1f-7c1a-9a32-700000000204";
export const INCOME_ENVELOPE_ID = "018f9df8-6c1f-7c1a-9a32-700000000304";
export const GOALS_CATEGORY_ID = "018f9df8-6c1f-7c1a-9a32-700000000105";
export const GOALS_SUBCATEGORY_ID = "018f9df8-6c1f-7c1a-9a32-700000000205";

export type BudgetCategory = {
  id: string;
  name: string;
};

export type BudgetSubcategory = {
  id: string;
  categoryId: string;
  name: string;
};

export type BudgetEnvelope = {
  id: string;
  subcategoryId: string;
  name: string;
  assignedAmount: string;
  spentAmount: string;
};

export type BudgetGoal = {
  id: string;
  envelopeId: string;
  targetAmount: string;
  deadlineDate: string;
  status: BudgetGoalStatus;
};

const categories = ref<BudgetCategory[]>([]);

const subcategories = ref<BudgetSubcategory[]>([]);

const envelopes = ref<BudgetEnvelope[]>([]);

const goals = ref<BudgetGoal[]>([]);

const movementCategoryOptions = computed(() =>
  envelopes.value
    .filter((envelope) => envelope.subcategoryId !== TRASH_SUBCATEGORY_ID)
    .map((envelope) => ({
      id: envelope.id,
      name: envelope.name,
    })),
);

export function applyBudgetTree(tree: BudgetTreeDto) {
  categories.value = tree.categories
    .slice()
    .sort((first, second) => first.sortOrder - second.sortOrder)
    .map((category) => ({
      id: category.id,
      name: category.name,
    }));
  subcategories.value = tree.subcategories
    .slice()
    .sort((first, second) => first.sortOrder - second.sortOrder)
    .map((subcategory) => ({
      id: subcategory.id,
      categoryId: subcategory.categoryId,
      name: subcategory.name,
    }));
  envelopes.value = tree.envelopes
    .slice()
    .sort((first, second) => first.sortOrder - second.sortOrder)
    .map((envelope) => ({
      id: envelope.id,
      subcategoryId: envelope.subcategoryId,
      name: envelope.name,
      assignedAmount: envelope.assignedAmount,
      spentAmount: "0",
    }));
  goals.value = (tree.goals ?? []).map((goal) => ({
    id: goal.id,
    envelopeId: goal.envelopeId,
    targetAmount: goal.targetAmount,
    deadlineDate: goal.deadlineDate,
    status: goal.status,
  }));
}

export function toBudgetTreeDto(): BudgetTreeDto {
  return {
    categories: categories.value.map((category, index) => ({
      id: category.id,
      name: category.name,
      sortOrder: index,
    })),
    subcategories: subcategories.value.map((subcategory, index) => ({
      id: subcategory.id,
      categoryId: subcategory.categoryId,
      name: subcategory.name,
      sortOrder: index,
    })),
    envelopes: envelopes.value.map((envelope, index) => ({
      id: envelope.id,
      subcategoryId: envelope.subcategoryId,
      name: envelope.name,
      assignedAmount: envelope.assignedAmount,
      sortOrder: index,
    })),
    goals: goals.value.map((goal) => ({
      id: goal.id,
      envelopeId: goal.envelopeId,
      targetAmount: goal.targetAmount,
      deadlineDate: goal.deadlineDate,
      status: goal.status,
    })),
  };
}

export function useBudgetTree() {
  return {
    categories,
    subcategories,
    envelopes,
    goals,
    movementCategoryOptions,
  };
}
