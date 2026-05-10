import { useQuery } from "@pinia/colada";
import { computed, ref, watch } from "vue";
import type { TreeTableExpandedKeys } from "primevue/treetable";

import { getBudgetTree, saveBudgetTree } from "@/common/api/ledger";
import {
  INCOME_CATEGORY_ID,
  INCOME_ENVELOPE_ID,
  INCOME_SUBCATEGORY_ID,
  TRASH_CATEGORY_ID,
  TRASH_SUBCATEGORY_ID,
  applyBudgetTree,
  toBudgetTreeDto,
  useBudgetTree,
  type BudgetEnvelope,
  type BudgetGoal,
} from "@/features/budget/model/budgetTree";
import {
  amountTone,
  budgetCurrency,
  formatDecimal,
  parseAmount,
} from "@/features/budget/model/amounts";
import { buildBudgetTreeNodes } from "@/features/budget/model/treeNodes";
import { useBudgetDragDrop } from "@/features/budget/composables/useBudgetDragDrop";
import { useBudgetGoalDrawer } from "@/features/budget/composables/useBudgetGoalDrawer";
import { useBudgetTreeEditor } from "@/features/budget/composables/useBudgetTreeEditor";

export function useBudgetWorkspace(period: () => string) {
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
    key: () => ["budget-tree", period()],
    query: () => getBudgetTree(period()),
  });

  const budgetEnvelopes = computed<BudgetEnvelope[]>(() =>
    envelopes.value.map((envelope) => ({ ...envelope })),
  );
  const goalByEnvelopeId = computed<Record<string, BudgetGoal>>(() =>
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
      budgetEnvelopes.value.find(
        (envelope) => envelope.id === INCOME_ENVELOPE_ID,
      )?.spentAmount ?? "0",
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
  const incomeLeftTone = computed(() => amountTone(incomeLeftToAssign.value));
  const visibleEnvelopeCount = computed(
    () =>
      activeBudgetEnvelopes.value.filter(
        (envelope) => envelope.id !== INCOME_ENVELOPE_ID,
      ).length,
  );
  const nodes = computed(() =>
    buildBudgetTreeNodes(
      categories.value,
      subcategories.value,
      activeBudgetEnvelopes.value,
      goalByEnvelopeId.value,
    ),
  );

  const editor = useBudgetTreeEditor({
    categories,
    subcategories,
    envelopes,
    goals,
    goalByEnvelopeId,
    expandedKeys,
    persistBudgetTree,
  });
  const goalDrawer = useBudgetGoalDrawer({
    budgetEnvelopes,
    goals,
    persistBudgetTree,
  });
  const dragDrop = useBudgetDragDrop({
    moveCategoryNear: editor.moveCategoryNear,
    moveEnvelope: editor.moveEnvelope,
    moveEnvelopeNear: editor.moveEnvelopeNear,
    moveSubcategory: editor.moveSubcategory,
    moveSubcategoryNear: editor.moveSubcategoryNear,
  });

  watch(
    () => budgetTreeQuery.data.value,
    (tree) => {
      if (tree) {
        applyBudgetTree(tree);
        editor.expandExistingBudgetTree();
      }
    },
    { immediate: true },
  );

  return {
    currency: budgetCurrency,
    expandedKeys,
    incomeLeftToAssign,
    incomeLeftTone,
    nodes,
    treeTablePassThrough: dragDrop.treeTablePassThrough,
    visibleEnvelopeCount,
    ...editor,
    ...goalDrawer,
  };

  async function persistBudgetTree() {
    const savedTree = await saveBudgetTree(period(), toBudgetTreeDto(period()));
    applyBudgetTree(savedTree);
  }
}
