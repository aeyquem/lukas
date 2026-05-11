<script setup lang="ts">
import { computed, ref } from "vue";

import AccountsPanel from "@/features/ledger/components/AccountsPanel.vue";
import AppHeader from "@/common/components/layout/AppHeader.vue";
import AppSidebar from "@/common/components/layout/AppSidebar.vue";
import BudgetWorkspace from "@/features/budget/views/BudgetWorkspace.vue";
import GoalsWorkspace from "@/features/goals/views/GoalsWorkspace.vue";
import MovementsWorkspace from "@/features/ledger/views/MovementsWorkspace.vue";
import ProjectionsWorkspace from "@/features/projections/views/ProjectionsWorkspace.vue";
import ReportsWorkspace from "@/features/reports/views/ReportsWorkspace.vue";
import TagsWorkspace from "@/features/tags/views/TagsWorkspace.vue";
import { useLedgerWorkspace } from "@/features/ledger/composables/useLedgerWorkspace";
import {
  workspaceSectionEyebrow,
  workspaceSectionTitle,
  type WorkspaceSection,
} from "@/common/workspace/sections";

const activeSection = ref<WorkspaceSection>("accounts");
const sectionTitle = computed(() => workspaceSectionTitle(activeSection.value));
const sectionEyebrow = computed(() =>
  workspaceSectionEyebrow(activeSection.value),
);

const {
  selectedPeriod,
  accountsQuery,
  accountMutation,
  transactionMutation,
  updateTransactionMutation,
  accounts,
  submitAccount,
  submitTransaction,
  updateTransaction,
} = useLedgerWorkspace();

function selectSection(section: WorkspaceSection) {
  activeSection.value = section;
}
</script>

<template>
  <div class="app-layout">
    <AppSidebar
      :active-section="activeSection"
      @select-section="selectSection"
    />

    <div class="app-workspace">
      <AppHeader
        v-model:period="selectedPeriod"
        :section-eyebrow="sectionEyebrow"
        :section-title="sectionTitle"
      />

      <main class="app-main">
        <AccountsPanel
          v-if="activeSection === 'accounts'"
          :accounts="accounts"
          :has-load-error="Boolean(accountsQuery.error.value)"
          :is-creating="accountMutation.isLoading.value"
          @submit-account="submitAccount"
        />

        <MovementsWorkspace
          v-else-if="activeSection === 'movements'"
          :accounts="accounts"
          :create-error="transactionMutation.error.value?.message"
          :create-transaction="submitTransaction"
          :has-accounts-load-error="Boolean(accountsQuery.error.value)"
          :is-creating="transactionMutation.isLoading.value"
          :is-updating="updateTransactionMutation.isLoading.value"
          :period="selectedPeriod"
          :update-error="updateTransactionMutation.error.value?.message"
          :update-transaction="updateTransaction"
        />
        <TagsWorkspace v-else-if="activeSection === 'tags'" />
        <BudgetWorkspace
          v-else-if="activeSection === 'budget'"
          :period="selectedPeriod"
        />
        <GoalsWorkspace
          v-else-if="activeSection === 'goals'"
          :period="selectedPeriod"
          @open-budget="selectSection('budget')"
        />
        <ReportsWorkspace
          v-else-if="activeSection === 'reports'"
          :period="selectedPeriod"
        />
        <ProjectionsWorkspace v-else :period="selectedPeriod" />
      </main>
    </div>
  </div>
</template>

<style scoped>
.app-layout {
  display: grid;
  grid-template-columns: var(--lu-size-sidebar) minmax(0, 1fr);
  min-block-size: 100dvh;
  background: var(--lu-color-background);
}

.app-workspace {
  min-inline-size: 0;
}

.app-main {
  display: grid;
  gap: var(--lu-space-4);
  inline-size: 100%;
  max-inline-size: var(--lu-size-main);
  min-inline-size: 0;
  padding: var(--lu-space-6);
}

.app-main > * {
  min-inline-size: 0;
}

@media (max-width: 900px) {
  .app-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 680px) {
  .app-main {
    padding: var(--lu-space-4);
  }
}
</style>
