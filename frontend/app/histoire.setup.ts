import type { App } from "vue";

import { installPrimeVue } from "@/app/primevue";
import "@/common/styles/main.css";

export function setupVue3({ app }: { app: App<Element> }) {
  installPrimeVue(app);
}

export function setupVanilla() {
  return;
}
