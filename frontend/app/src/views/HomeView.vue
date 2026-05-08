<script setup lang="ts">
import { computed, ref } from "vue";

import AccountsPanel from "@/components/ledger/AccountsPanel.vue";
import AppHeader from "@/components/layout/AppHeader.vue";
import AppSidebar from "@/components/layout/AppSidebar.vue";
import BudgetWorkspace from "@/components/workspace/BudgetWorkspace.vue";
import GoalsWorkspace from "@/components/workspace/GoalsWorkspace.vue";
import MovementsWorkspace from "@/components/workspace/MovementsWorkspace.vue";
import ProjectionsWorkspace from "@/components/workspace/ProjectionsWorkspace.vue";
import ReportsWorkspace from "@/components/workspace/ReportsWorkspace.vue";
import { useLedgerWorkspace } from "@/composables/useLedgerWorkspace";
import {
  workspaceSectionEyebrow,
  workspaceSectionTitle,
  type WorkspaceSection,
} from "@/workspace/sections";

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
          :update-error="updateTransactionMutation.error.value?.message"
          :update-transaction="updateTransaction"
        />
        <BudgetWorkspace v-else-if="activeSection === 'budget'" />
        <GoalsWorkspace v-else-if="activeSection === 'goals'" />
        <ReportsWorkspace v-else-if="activeSection === 'reports'" />
        <ProjectionsWorkspace v-else />
      </main>
    </div>
  </div>
</template>

<style scoped>
.app-layout {
  display: grid;
  grid-template-columns: 17rem minmax(0, 1fr);
  min-block-size: 100dvh;
  background:
    linear-gradient(180deg, rgb(255 255 255 / 0.42), transparent 16rem),
    var(--color-background);
}

.app-workspace {
  min-inline-size: 0;
}

.app-main {
  display: grid;
  gap: var(--space-4);
  max-inline-size: 76rem;
  padding: var(--space-6);
}

@media (max-width: 900px) {
  .app-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 680px) {
  .app-main {
    padding: var(--space-4);
  }
}
</style>
