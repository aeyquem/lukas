<script setup lang="ts">
import { computed, ref } from "vue";

import AccountsPanel from "../components/ledger/AccountsPanel.vue";
import MonthlySummary from "../components/ledger/MonthlySummary.vue";
import TransactionPanel from "../components/ledger/TransactionPanel.vue";
import TransactionsPanel from "../components/ledger/TransactionsPanel.vue";
import AppHeader from "../components/layout/AppHeader.vue";
import AppSidebar from "../components/layout/AppSidebar.vue";
import BudgetWorkspace from "../components/workspace/BudgetWorkspace.vue";
import ProjectionsWorkspace from "../components/workspace/ProjectionsWorkspace.vue";
import ReportsWorkspace from "../components/workspace/ReportsWorkspace.vue";
import { useLedgerWorkspace } from "../composables/useLedgerWorkspace";
import {
  workspaceSectionEyebrow,
  workspaceSectionTitle,
  type WorkspaceSection,
} from "../workspace/sections";

const activeSection = ref<WorkspaceSection>("accounts");
const sectionTitle = computed(() => workspaceSectionTitle(activeSection.value));
const sectionEyebrow = computed(() =>
  workspaceSectionEyebrow(activeSection.value),
);

const {
  selectedPeriod,
  accountsQuery,
  transactionsQuery,
  accountMutation,
  transactionMutation,
  accounts,
  transactions,
  cashflow,
  submitAccount,
  submitTransaction,
} = useLedgerWorkspace();

function selectSection(section: WorkspaceSection) {
  activeSection.value = section;
}
</script>

<template>
  <div class="app-layout">
    <AppSidebar
      :accounts="accounts"
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
        <template v-if="activeSection === 'accounts'">
          <MonthlySummary :cashflow="cashflow" />

          <section class="workspace-grid">
            <AccountsPanel
              :accounts="accounts"
              :has-load-error="Boolean(accountsQuery.error.value)"
              :is-creating="accountMutation.isLoading.value"
              @submit-account="submitAccount"
            />

            <TransactionPanel
              :accounts="accounts"
              :is-creating="transactionMutation.isLoading.value"
              @submit-transaction="submitTransaction"
            />

            <TransactionsPanel
              :has-load-error="Boolean(transactionsQuery.error.value)"
              :transactions="transactions"
            />
          </section>
        </template>

        <BudgetWorkspace v-else-if="activeSection === 'budget'" />
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

.workspace-grid {
  display: grid;
  grid-template-columns: minmax(16rem, 0.9fr) minmax(18rem, 1fr);
  gap: var(--space-4);
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

  .workspace-grid {
    grid-template-columns: 1fr;
  }
}
</style>
