<script setup lang="ts">
import DataTable from "primevue/datatable";

defineOptions({
  inheritAttrs: false,
});
</script>

<template>
  <DataTable class="base-data-table" v-bind="$attrs">
    <template
      v-for="(_, slotName) in $slots"
      #[slotName]="slotProps"
      :key="slotName"
    >
      <slot :name="slotName" v-bind="slotProps ?? {}" />
    </template>
  </DataTable>
</template>

<style scoped>
.base-data-table {
  max-inline-size: 100%;
  min-inline-size: 0;
}

.base-data-table :deep(.p-datatable-table-container) {
  overflow-x: auto;
}
</style>
