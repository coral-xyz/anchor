const { description } = require("../../package");

module.exports = {
  base: "/anchor/",
  /**
   * Ref：https://v1.vuepress.vuejs.org/config/#title
   */
  title: "Anchor",
  /**
   * Ref：https://v1.vuepress.vuejs.org/config/#description
   */
  description: description,

  /**
   * Extra tags to be injected to the page HTML `<head>`
   *
   * ref：https://v1.vuepress.vuejs.org/config/#head
   */
  head: [
    ["meta", { name: "theme-color", content: "#3eaf7c" }],
    ["meta", { name: "apple-mobile-web-app-capable", content: "yes" }],
    [
      "meta",
      { name: "apple-mobile-web-app-status-bar-style", content: "black" },
    ],
  ],

  /**
   * Theme configuration, here is the default theme configuration for VuePress.
   *
   * ref：https://v1.vuepress.vuejs.org/theme/default-theme-config.html
   */
  themeConfig: {
    repo: "",
    editLinks: false,
    docsDir: "",
    editLinkText: "",
    lastUpdated: false,
    sidebar: [
      {
        collapsable: false,
        title: "Getting Started",
        children: [
          "/getting-started/introduction",
          "/getting-started/installation",
        ],
      },
      {
        collapsable: false,
        title: "Programs on Solana",
        children: [
          "/tutorials/tutorial-0",
          "/tutorials/tutorial-1",
          "/tutorials/tutorial-2",
          "/tutorials/tutorial-3",
          "/tutorials/tutorial-4",
          "/tutorials/tutorial-5",
        ],
      },
      {
        collapsable: false,
        title: "CLI",
        children: [
          "/cli/commands",
        ],
      },
    ],

    nav: [{ text: "GitHub", link: "https://github.com/project-serum/anchor" }],
  },

  /**
   * Apply plugins，ref：https://v1.vuepress.vuejs.org/zh/plugin/
   */
  plugins: [
    "dehydrate",
    "@vuepress/plugin-back-to-top",
    "@vuepress/plugin-medium-zoom",
  ],
};
