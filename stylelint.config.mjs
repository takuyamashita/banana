// CSS の決まり。層(packages/ui/src/styles/ のファイル)ごとに、書いてよいものを絞る。
// ファイルをまたぐ決まり(層の順・変数の定義・TSX との対応)は packages/ui/scripts/styles.ts が見る
const colorFunctions = ["rgb", "rgba", "hsl", "hsla", "hwb", "lab", "lch", "oklab", "oklch", "color", "color-mix"];

/// 値はトークン(var(--…))で書かせる。長さの単位を書けるのは tokens.css だけ
const tokenOnlyValues = {
  "color-no-hex": true,
  "color-named": "never",
  "function-disallowed-list": colorFunctions,
  "declaration-property-unit-allowed-list": {
    "/^(margin|padding|gap|row-gap|column-gap|inset|top|right|bottom|left|border|outline|font-size|line-height|letter-spacing)/":
      [],
    "/^(width|height|min-width|min-height|max-width|max-height)$/": ["%"],
  },
};

export default {
  extends: ["stylelint-config-standard"],
  ignoreFiles: ["**/node_modules/**", "**/dist/**"],
  rules: {
    // 勝ち負けは層で決める
    "declaration-no-important": true,
    "selector-max-id": 0,
    "selector-class-pattern": ["^[a-z][a-z0-9]*(-[a-z0-9]+)*$", { message: "クラス名は小文字とハイフン(p-t-1)" }],
    "custom-property-pattern": [
      "^(color|space|radius|border|font|size|opacity)-[a-z0-9-]+$",
      { message: "変数は種類から始める(--color-…・--space-…など)" },
    ],
    "import-notation": "string",
    // 層を付けるのと読み込むのは index.css だけ
    "at-rule-disallowed-list": ["layer", "import"],
  },
  overrides: [
    {
      files: ["**/styles/index.css"],
      rules: {
        "at-rule-disallowed-list": null,
        "at-rule-allowed-list": ["layer", "import"],
        "selector-disallowed-list": [["/./"], { message: "index.css は層の宣言と読み込みだけ" }],
      },
    },
    {
      files: ["**/styles/reset.css"],
      rules: {
        "selector-max-class": 0,
        "function-disallowed-list": ["var", ...colorFunctions],
        "color-no-hex": true,
        "color-named": "never",
      },
    },
    {
      files: ["**/styles/tokens.css"],
      rules: {
        "selector-disallowed-list": [["/^(?!:root$)/"], { message: "tokens.css は :root にだけ書く" }],
        "property-allowed-list": [["/^--/", "color-scheme"], { message: "tokens.css は変数だけ" }],
        "at-rule-allowed-list": ["media"],
      },
    },
    {
      files: ["**/styles/base.css"],
      rules: {
        ...tokenOnlyValues,
        "selector-max-class": 0,
        "selector-max-attribute": 0,
        "selector-max-compound-selectors": 1,
        "at-rule-allowed-list": [],
      },
    },
    {
      files: ["**/styles/utilities.css"],
      rules: {
        ...tokenOnlyValues,
        "selector-max-class": 1,
        "selector-max-type": 0,
        "selector-max-universal": 0,
        "selector-max-attribute": 0,
        "selector-max-pseudo-class": 0,
        "selector-pseudo-element-disallowed-list": ["/./"],
        "selector-max-combinators": 0,
        "selector-disallowed-list": [["/,/"], { message: "1つのクラスに1つのルール(まとめない)" }],
        "declaration-block-single-line-max-declarations": 1,
        "at-rule-allowed-list": [],
      },
    },
  ],
};
