import { config } from "@vue/test-utils";
import PrimeVue from "primevue/config";

import { primeVueOptions } from "@/primevue";

config.global.plugins = [[PrimeVue, primeVueOptions]];

Object.defineProperty(window, "matchMedia", {
  value: (query: string) => ({
    addEventListener: () => undefined,
    addListener: () => undefined,
    dispatchEvent: () => false,
    matches: false,
    media: query,
    onchange: null,
    removeEventListener: () => undefined,
    removeListener: () => undefined,
  }),
  writable: true,
});
