<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { FilterOperator, FilterMatchMode } from "@primevue/core/api";
import Column from "primevue/column";
import DataTable from "primevue/datatable";
import InputText from "primevue/inputtext";
import MultiSelect from "primevue/multiselect";
import Select from "primevue/select";
import type { DataTableFilterMeta } from "primevue/datatable";

import { formatDate, formatMoney } from "@/formatters";
import {
  directionLabel,
  type LedgerMovement,
  type MovementDirection,
} from "@/ledger/movements";

type DisplayMovement = LedgerMovement & {
  displayAmount: string;
  displayCurrency: string;
  signedDisplayAmount: string;
  directionLabel: string;
};

type GroupField = "none" | "accountName";

type GroupOption = {
  label: string;
  value: GroupField;
};

type AccountOption = {
  label: string;
  value: string;
};

const props = withDefaults(
  defineProps<{
    movements: LedgerMovement[];
    showOriginalCurrency?: boolean;
    groupBy?: GroupField;
  }>(),
  {
    groupBy: "none",
    showOriginalCurrency: false,
  },
);

const selectedAccountNames = ref<string[]>([]);
const selectedGroupBy = ref<GroupField>(props.groupBy);
const rowsPerPage = ref(20);
const filters = ref<DataTableFilterMeta>({
  notes: {
    operator: FilterOperator.AND,
    constraints: [{ value: "", matchMode: FilterMatchMode.CONTAINS }],
  },
});

const groupOptions: GroupOption[] = [
  { label: "Sin agrupar", value: "none" },
  { label: "Cuenta", value: "accountName" },
];

const rowsPerPageOptions = [10, 20, 50, 100, 250, 500];

const accountOptions = computed<AccountOption[]>(() =>
  Array.from(new Set(props.movements.map((movement) => movement.accountName)))
    .sort((firstAccount, secondAccount) =>
      firstAccount.localeCompare(secondAccount, "es-CL"),
    )
    .map((accountName) => ({
      label: accountName,
      value: accountName,
    })),
);

watch(
  accountOptions,
  (options) => {
    const accountNames = options.map((account) => account.value);
    const stillAvailable = selectedAccountNames.value.filter((accountName) =>
      accountNames.includes(accountName),
    );
    const newlyAvailable = accountNames.filter(
      (accountName) => !stillAvailable.includes(accountName),
    );

    selectedAccountNames.value = [...stillAvailable, ...newlyAvailable];
  },
  { immediate: true },
);

const selectedAccounts = computed<string[]>(() => {
  return selectedAccountNames.value;
});

const activeGroupBy = computed(() =>
  selectedGroupBy.value === "none" ? undefined : selectedGroupBy.value,
);

const sortField = computed(() => activeGroupBy.value ?? "date");
const sortOrder = computed(() => (activeGroupBy.value ? 1 : -1));

const displayMovements = computed<DisplayMovement[]>(() =>
  props.movements
    .map((movement) => {
      const displayAmount = props.showOriginalCurrency
        ? movement.originalAmount
        : movement.baseAmount;
      const displayCurrency = props.showOriginalCurrency
        ? movement.originalCurrency
        : movement.baseCurrency;

      return {
        ...movement,
        directionLabel: directionLabel(movement.direction),
        displayAmount,
        displayCurrency,
        signedDisplayAmount: signedAmount(
          movement.direction,
          displayAmount,
          displayCurrency,
        ),
      };
    })
    .filter((movement) =>
      selectedAccounts.value.includes(movement.accountName),
    ),
);

function signedAmount(
  direction: MovementDirection,
  amount: string,
  currency: string,
): string {
  const sign = direction === "out" ? "-" : "+";
  return `${sign}${formatMoney(amount, currency)}`;
}

function groupHeading(data: DisplayMovement): string {
  if (!activeGroupBy.value) {
    return "";
  }

  return String(data[activeGroupBy.value]);
}
</script>

<template>
  <DataTable
    v-model:filters="filters"
    class="ledger-movements-table"
    column-resize-mode="fit"
    data-key="id"
    :group-rows-by="activeGroupBy"
    paginator
    removable-sort
    reorderable-columns
    resizable-columns
    filter-display="menu"
    :row-group-mode="activeGroupBy ? 'subheader' : undefined"
    :rows="rowsPerPage"
    :rows-per-page-options="rowsPerPageOptions"
    show-gridlines
    :sort-field="sortField"
    :sort-order="sortOrder"
    :value="displayMovements"
  >
    <template #header>
      <div class="table-toolbar">
        <div class="table-filter">
          <label for="movement-table-accounts">Cuentas</label>
          <MultiSelect
            id="movement-table-accounts"
            v-model="selectedAccountNames"
            display="chip"
            :max-selected-labels="accountOptions.length - 1"
            option-label="label"
            option-value="value"
            filter
            :options="accountOptions"
            placeholder="Seleccionar"
            selected-items-label="Todas las cuentas"
          />
        </div>

        <div class="table-filter table-filter--group">
          <label for="movement-table-group">Agrupar</label>
          <Select
            id="movement-table-group"
            v-model="selectedGroupBy"
            option-label="label"
            option-value="value"
            :options="groupOptions"
          />
        </div>

        <div class="table-filter table-filter--rows">
          <label for="movement-table-rows">Filas</label>
          <Select
            id="movement-table-rows"
            v-model="rowsPerPage"
            :options="rowsPerPageOptions"
          />
        </div>
      </div>
    </template>

    <template #groupheader="{ data }">
      <span class="group-heading">{{ groupHeading(data) }}</span>
    </template>

    <Column field="date" header="Fecha" sortable style="width: 9.5rem">
      <template #body="{ data }">
        {{ formatDate(data.date) }}
      </template>
    </Column>

    <Column field="accountName" header="Cuenta" sortable style="width: 14rem">
      <template #body="{ data }">
        <strong>{{ data.accountName }}</strong>
      </template>
    </Column>

    <Column
      field="description"
      header="Detalle"
      sortable
      style="min-width: 16rem"
    />

    <Column
      field="notes"
      header="Notas"
      style="min-width: 14rem"
      filter-field="notes"
      :filter-match-mode="FilterMatchMode.CONTAINS"
      :show-add-button="false"
      :show-filter-match-modes="false"
      :show-filter-operator="false"
    >
      <template #body="{ data }">
        <span class="notes">{{ data.notes || "Sin notas" }}</span>
      </template>
      <template #filter="{ filterModel }">
        <InputText
          v-model="filterModel.value"
          class="notes-filter"
          placeholder="Buscar notas"
          type="text"
        />
      </template>
    </Column>

    <Column field="directionLabel" header="Tipo" sortable style="width: 10rem">
      <template #body="{ data }">
        <span class="direction" :data-direction="data.direction">
          {{ data.directionLabel }}
        </span>
      </template>
    </Column>

    <Column
      field="displayCurrency"
      header="Moneda"
      sortable
      style="width: 8rem"
    />

    <Column
      field="signedDisplayAmount"
      header="Valor"
      sortable
      style="width: 12rem"
    >
      <template #body="{ data }">
        <span class="numeric" :data-direction="data.direction">
          {{ data.signedDisplayAmount }}
        </span>
      </template>
    </Column>

    <template #empty>Sin movimientos para mostrar.</template>
  </DataTable>
</template>

<style scoped>
.table-toolbar {
  display: grid;
  grid-template-columns:
    minmax(14rem, 1.2fr) minmax(11rem, 0.65fr)
    minmax(7rem, 0.35fr);
  gap: var(--space-2);
}

.table-filter {
  display: grid;
  align-content: start;
  gap: var(--space-1);
}

.table-filter label {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-sm);
  font-weight: 700;
}

.table-filter :deep(.p-multiselect),
.table-filter :deep(.p-select) {
  inline-size: 100%;
}

.table-filter--rows {
  justify-self: end;
  inline-size: min(8rem, 100%);
}

.group-heading {
  color: var(--color-primary-strong);
  font-size: var(--font-size-sm);
  font-weight: 800;
}

.direction {
  display: inline-flex;
  align-items: center;
  min-block-size: 1.65rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-round);
  padding-inline: var(--space-3);
  font-size: var(--font-size-xs);
  font-weight: 800;
}

.direction[data-direction="in"] {
  border-color: color-mix(in srgb, var(--color-success) 35%, transparent);
  background: color-mix(in srgb, var(--color-success) 10%, transparent);
  color: var(--color-primary-strong);
}

.direction[data-direction="out"] {
  border-color: color-mix(in srgb, var(--color-danger) 35%, transparent);
  background: color-mix(in srgb, var(--color-danger) 8%, transparent);
  color: var(--color-danger-strong);
}

.numeric {
  font-family: var(--font-mono);
  font-weight: 700;
  white-space: nowrap;
}

.notes {
  color: var(--color-muted-foreground);
  overflow-wrap: anywhere;
}

.notes-filter {
  inline-size: 100%;
}

.numeric[data-direction="in"] {
  color: var(--color-primary-strong);
}

.numeric[data-direction="out"] {
  color: var(--color-danger-strong);
}

:deep(.p-datatable-header) {
  border-start-start-radius: var(--radius-lg);
  border-start-end-radius: var(--radius-lg);
}

:deep(.p-datatable-table) {
  font-size: var(--font-size-sm);
}

:deep(.p-datatable-column-title),
:deep(.p-paginator) {
  font-size: var(--font-size-sm);
}

:deep(.p-datatable-thead > tr > th) {
  white-space: nowrap;
}

@media (max-width: 680px) {
  .table-toolbar {
    grid-template-columns: 1fr;
  }

  .table-filter--rows {
    justify-self: stretch;
    inline-size: 100%;
  }
}
</style>
