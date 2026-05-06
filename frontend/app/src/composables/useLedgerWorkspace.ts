import { useMutation, useQuery, useQueryCache } from "@pinia/colada";
import { computed, ref } from "vue";

import {
  createAccount,
  createManualTransaction,
  getAccounts,
  getMonthlyCashflow,
  getTransactions,
  type CreateAccountRequest,
  type CreateManualTransactionRequest,
} from "../api/ledger";
import { currentPeriod } from "../formatters";

export function useLedgerWorkspace() {
  const queryCache = useQueryCache();
  const selectedPeriod = ref(currentPeriod());

  const accountsQuery = useQuery({
    key: ["accounts"],
    query: getAccounts,
  });

  const transactionsQuery = useQuery({
    key: () => ["transactions", selectedPeriod.value],
    query: () => getTransactions(selectedPeriod.value),
  });

  const cashflowQuery = useQuery({
    key: () => ["cashflow", selectedPeriod.value],
    query: () => getMonthlyCashflow(selectedPeriod.value),
  });

  const accountMutation = useMutation({
    mutation: createAccount,
    async onSettled() {
      await invalidateLedgerQueries();
    },
  });

  const transactionMutation = useMutation({
    mutation: createManualTransaction,
    async onSettled() {
      await invalidateLedgerQueries();
    },
  });

  const accounts = computed(() => accountsQuery.data.value ?? []);
  const transactions = computed(() => transactionsQuery.data.value ?? []);
  const cashflow = computed(() => cashflowQuery.data.value);

  async function invalidateLedgerQueries() {
    await queryCache.invalidateQueries({ key: ["accounts"] });
    await queryCache.invalidateQueries({ key: ["transactions"] });
    await queryCache.invalidateQueries({ key: ["cashflow"] });
  }

  function submitAccount(request: CreateAccountRequest) {
    accountMutation.mutate(request);
  }

  function submitTransaction(request: CreateManualTransactionRequest) {
    transactionMutation.mutate(request);
  }

  return {
    selectedPeriod,
    accountsQuery,
    transactionsQuery,
    cashflowQuery,
    accountMutation,
    transactionMutation,
    accounts,
    transactions,
    cashflow,
    submitAccount,
    submitTransaction,
  };
}
