import { computed, ref } from "vue";

import type { BudgetTreeDto } from "@/api/ledger";

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

const categories = ref<BudgetCategory[]>([]);

const subcategories = ref<BudgetSubcategory[]>([]);

const envelopes = ref<BudgetEnvelope[]>([]);

const movementCategoryOptions = computed(() =>
  envelopes.value.map((envelope) => ({
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
  };
}

export function useBudgetTree() {
  return {
    categories,
    subcategories,
    envelopes,
    movementCategoryOptions,
  };
}
