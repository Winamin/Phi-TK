/* eslint-env node */
require("@rushstack/eslint-patch/modern-module-resolution");

module.exports = {
  root: true,
  extends: [
    "plugin:vue/vue3-essential",
    "eslint:recommended",
    "@vue/eslint-config-typescript",
  ],
  parserOptions: {
    ecmaVersion: "latest",
  },
  rules: {
    // mdui's Web Components use the native `slot="name"` attribute, which this rule
    // flags as the deprecated Vue 2 slot syntax.
    "vue/no-deprecated-slot-attribute": "off",
  },
};
