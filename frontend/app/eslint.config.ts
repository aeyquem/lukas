import pluginVue from "eslint-plugin-vue";
import vueTsEslintConfig from "@vue/eslint-config-typescript";
import skipFormatting from "@vue/eslint-config-prettier/skip-formatting";

export default [
  ...pluginVue.configs["flat/recommended"],
  ...vueTsEslintConfig(),
  skipFormatting,
  {
    ignores: ["dist", "coverage", "src/api/generated"],
  },
];
