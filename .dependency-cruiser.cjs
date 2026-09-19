// フロントエンドの依存の向き。上から下へだけ依存する(下は上を知らない)。
//
//   apps/web:  main.tsx → app.tsx・router.tsx → routes/ → features/・layout/ → lib/
//   packages:  apps → packages/ui・packages/api-client(packages は apps を知らない。ui と api-client は互いを知らない)
//   e2e:       packages/api-client だけを使う(画面のコードには触れない)
//
// 画面の CSS は main.tsx が @platform/ui/styles.css を1回だけ読み込む(層の順は packages/ui/src/styles/index.css)
const web = "^frontend/apps/web/src/";

/** @type {import("dependency-cruiser").IConfiguration} */
module.exports = {
  forbidden: [
    {
      name: "no-circular",
      severity: "error",
      comment: "循環した依存は、どちらが上かが決まらない",
      from: {},
      to: { circular: true },
    },
    {
      name: "not-to-unresolvable",
      severity: "error",
      comment: "解決できない import",
      from: {},
      to: { couldNotResolve: true },
    },
    {
      name: "features-are-independent",
      severity: "error",
      comment: "features/ の間では import しない。共有するものは lib/ か packages/ に置く",
      from: { path: `${web}features/([^/]+)/` },
      to: { path: `${web}features/`, pathNot: `${web}features/$1/` },
    },
    {
      name: "features-not-to-shell",
      severity: "error",
      comment: "features/ は画面の枠(routes/・layout/・アプリの組み立て)を知らない。URL や枠からは props で受け取る",
      from: { path: `${web}features/` },
      to: { path: `${web}(routes/|layout/|app\\.tsx|router\\.tsx|main\\.tsx|routeTree\\.gen\\.ts)` },
    },
    {
      name: "layout-not-to-features",
      severity: "error",
      comment: "layout/ は描くだけの枠。features/ や routes/ を知らない",
      from: { path: `${web}layout/` },
      to: { path: `${web}(features/|routes/|app\\.tsx|router\\.tsx|main\\.tsx|routeTree\\.gen\\.ts)` },
    },
    {
      name: "lib-is-bottom",
      severity: "error",
      comment: "lib/ は一番下。ルートに渡す context の型も lib/ に置く",
      from: { path: `${web}lib/` },
      to: { path: `${web}(features/|routes/|layout/|app\\.tsx|main\\.tsx|routeTree\\.gen\\.ts|router\\.tsx)` },
    },
    {
      name: "routes-only-from-route-tree",
      severity: "error",
      comment: "ルートのファイルを読み込むのは、生成されたルートの木だけ",
      from: { pathNot: `${web}(routeTree\\.gen\\.ts|routes/)` },
      to: { path: `${web}routes/` },
    },
    {
      name: "packages-not-to-apps",
      severity: "error",
      comment: "packages/ は apps/ を知らない",
      from: { path: "^frontend/packages/" },
      to: { path: "^frontend/apps/" },
    },
    {
      name: "packages-are-independent",
      severity: "error",
      comment: "packages/ui と packages/api-client は互いを知らない(ui は画面の見た目だけ、api-client は API だけ)",
      from: { path: "^frontend/packages/([^/]+)/" },
      to: { path: "^frontend/packages/", pathNot: "^frontend/packages/$1/" },
    },
    {
      name: "e2e-not-to-app-code",
      severity: "error",
      comment: "E2E は画面のコードに触れず、ブラウザと API だけで確かめる",
      from: { path: "^e2e/" },
      to: { path: "^frontend/(apps/|packages/ui/)" },
    },
    {
      name: "css-only-from-entry",
      severity: "error",
      comment: "CSS は main.tsx が @platform/ui/styles.css を1回だけ読み込む(層に入らない CSS はどの層にも勝つ)",
      from: { pathNot: `${web}main\\.tsx$` },
      to: { path: "\\.css$" },
    },
    {
      name: "css-entry-is-ui-styles",
      severity: "error",
      comment: "読み込む CSS は packages/ui の入口だけ",
      from: {},
      to: { path: "\\.css$", pathNot: "^frontend/packages/ui/src/styles/index\\.css$" },
    },
    {
      name: "no-orphans",
      severity: "error",
      comment: "どこからも使われていないファイル",
      from: {
        orphan: true,
        pathNot: [
          "\\.(test|spec)\\.tsx?$",
          "\\.d\\.ts$",
          "(^|/)(vite|vitest|playwright)\\.config\\.ts$",
          "/test-setup\\.ts$",
          "/test-utils\\.tsx$",
          "^frontend/packages/ui/scripts/",
          `${web}main\\.tsx$`,
        ],
      },
      to: {},
    },
  ],
  options: {
    doNotFollow: { path: "node_modules" },
    exclude: { path: ["/gen/", "node_modules", "/dist/", "playwright-report", "test-results"] },
    // TypeScript 7 には dependency-cruiser が使う API がまだないので、swc で読む
    parser: "swc",
    tsPreCompilationDeps: true,
    // ワークスペースのパッケージ(@platform/*)は package.json の exports で src/ を指す
    enhancedResolveOptions: {
      exportsFields: ["exports"],
      conditionNames: ["import", "default"],
      extensions: [".ts", ".tsx", ".js"],
      mainFields: ["module", "main"],
    },
  },
};
