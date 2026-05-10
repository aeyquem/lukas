import { computed, ref, type ComputedRef, type Ref } from "vue";

import type { BudgetEnvelope, BudgetGoal } from "@/features/budget/model/budgetTree";
import {
  formatDecimal,
  normalizedAmount,
  suggestedContribution,
} from "@/features/budget/model/amounts";

type BudgetGoalDrawerOptions = {
  budgetEnvelopes: ComputedRef<BudgetEnvelope[]>;
  goals: Ref<BudgetGoal[]>;
  persistBudgetTree: () => Promise<void>;
};

export function useBudgetGoalDrawer(options: BudgetGoalDrawerOptions) {
  const { budgetEnvelopes, goals, persistBudgetTree } = options;
  const selectedGoalId = ref("");
  const goalForm = ref({
    targetAmount: "0",
    deadlineDate: "",
    status: "active" as "" | "active" | "completed" | "archived",
  });

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

  function openGoalDetails(envelopeId: string) {
    const goal = goals.value.find(
      (candidate) => candidate.envelopeId === envelopeId,
    );

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

  return {
    goalForm,
    isGoalDrawerOpen,
    selectedGoalEnvelope,
    suggestedGoalContribution,
    archiveGoal,
    closeGoalDetails,
    completeGoal,
    openGoalDetails,
    saveGoalDetails,
  };
}
