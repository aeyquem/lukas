import { PiniaColada } from "@pinia/colada";
import { createPinia } from "pinia";
import { createApp } from "vue";

import App from "@/app/App.vue";
import { installPrimeVue } from "@/app/primevue";
import { router } from "@/app/router";
import "@/common/styles/main.css";

const app = createApp(App);

installPrimeVue(app)
  .use(createPinia())
  .use(PiniaColada)
  .use(router)
  .mount("#app");
