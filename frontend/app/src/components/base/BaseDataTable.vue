<script setup lang="ts">
import DataTable from "primevue/datatable";

defineOptions({
  inheritAttrs: false,
});
</script>

<template>
  <DataTable v-bind="$attrs">
    <template
      v-for="(_, slotName) in $slots"
      #[slotName]="slotProps"
      :key="slotName"
    >
      <slot :name="slotName" v-bind="slotProps ?? {}" />
    </template>
  </DataTable>
</template>
