import { PiniaColada } from "@pinia/colada";
import { createPinia } from "pinia";
import { createApp } from "vue";

import App from "./App.vue";
import { installPrimeVue } from "./primevue";
import { router } from "./router";
import "./styles/main.css";

const app = createApp(App);

installPrimeVue(app)
  .use(createPinia())
  .use(PiniaColada)
  .use(router)
  .mount("#app");
