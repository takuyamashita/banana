// CSS の決まりのうち、stylelint では見られないものを確かめる(ファイルをまたぐもの・TSX との対応)。
//   node scripts/styles.ts        確かめる(mise run lint:css)
//   node scripts/styles.ts --gen  utilities.css からクラス名の型(src/utilities.gen.ts)を作る(mise run gen:styles)
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";

const ui = join(import.meta.dirname, "..");
const frontend = join(ui, "..", "..");
const stylesDir = join(ui, "src", "styles");
const genFile = join(ui, "src", "utilities.gen.ts");

/// 層の順。index.css の @layer はこのとおりに並べる
const LAYERS = ["vendor", "reset", "tokens", "base", "utilities"];

const read = (file: string) => readFileSync(file, "utf8");
const withoutComments = (css: string) => css.replace(/\/\*[\s\S]*?\*\//g, "");

function walk(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return entry.name === "node_modules" || entry.name === "dist" ? [] : walk(path);
    return [path];
  });
}

const SIDES = ["top", "right", "bottom", "left"];

/// プロパティが書き換える先(省略形は個々のプロパティに開く)。同じ先を持つクラスを1つの要素に並べると、
/// どちらが勝つかが utilities.css の並び順で決まってしまう
function longhands(property: string): string[] {
  const box = /^(margin|padding)(?:-(block|inline|top|right|bottom|left))?$/.exec(property);
  if (box) {
    const axis = box[2];
    const sides =
      axis === undefined
        ? SIDES
        : axis === "block"
          ? ["top", "bottom"]
          : axis === "inline"
            ? ["left", "right"]
            : [axis];
    return sides.map((side) => `${box[1]}-${side}`);
  }
  if (property === "border") return SIDES.map((side) => `border-${side}`);
  if (property === "gap") return ["row-gap", "column-gap"];
  if (property === "background-color") return ["background"];
  return [property];
}

/// utilities.css のクラスと、それぞれが書き換えるプロパティ(並び順のまま)
function utilityRules(): { name: string; properties: string[] }[] {
  const css = withoutComments(read(join(stylesDir, "utilities.css")));
  return [...css.matchAll(/^\.([a-z0-9-]+)\s*\{([^}]*)\}/gm)].map((m) => ({
    name: m[1] ?? "",
    properties: [
      ...new Set(
        (m[2] ?? "")
          .split(";")
          .map((declaration) => declaration.split(":")[0]?.trim() ?? "")
          .filter((property) => property !== "")
          .flatMap(longhands),
      ),
    ],
  }));
}

const utilityNames = () => utilityRules().map((rule) => rule.name);

function generated(): string {
  return [
    "// utilities.css から node scripts/styles.ts --gen で生成する。手で書き換えない",
    "// クラス名と、そのクラスが書き換えるプロパティ",
    "export const utilities = {",
    ...utilityRules().map(({ name, properties }) => `  "${name}": [${properties.map((p) => `"${p}"`).join(", ")}],`),
    "} as const;",
    "",
    "export type Utility = keyof typeof utilities;",
    "",
  ].join("\n");
}

if (process.argv.includes("--gen")) {
  writeFileSync(genFile, generated());
  process.exit(0);
}

const errors: string[] = [];
const fail = (message: string) => errors.push(message);

// 1. 入口(index.css)の層の順と読み込み。styles/ のファイルはすべて、層を付けて1回だけ読み込む
const index = withoutComments(read(join(stylesDir, "index.css")));
const declared = /@layer\s+([^;{]+);/
  .exec(index)?.[1]
  ?.split(",")
  .map((s) => s.trim());
if (declared?.join(",") !== LAYERS.join(",")) {
  fail(`index.css: 層の順は @layer ${LAYERS.join(", ")}; にする(今は ${declared?.join(", ") ?? "宣言なし"})`);
}
const imports = [...index.matchAll(/@import\s+"\.\/([^"]+)"\s+layer\(([a-z]+)\)\s*;/g)].map((m) => ({
  file: m[1] ?? "",
  layer: m[2] ?? "",
}));
const importedLayers = imports.map((i) => i.layer).filter((l) => l !== "vendor");
if (importedLayers.join(",") !== LAYERS.filter((l) => l !== "vendor").join(",")) {
  fail(`index.css: 読み込みは層の順に1つずつ(今は ${importedLayers.join(", ")})`);
}
for (const { file, layer } of imports) {
  if (layer !== "vendor" && file !== `${layer}.css`) fail(`index.css: ${file} は ${layer}.css という名前にする`);
}
const cssFiles = walk(join(frontend)).filter((f) => f.endsWith(".css"));
for (const file of cssFiles) {
  const rel = relative(stylesDir, file);
  if (rel.startsWith("..")) {
    fail(`${relative(frontend, file)}: CSS は packages/ui/src/styles/ にだけ置く(層に入らない CSS はどの層にも勝つ)`);
  } else if (rel !== "index.css" && !imports.some((i) => i.file === rel)) {
    fail(`${rel}: index.css から層を付けて読み込まれていない`);
  }
}

// 2. 生成した型が utilities.css と合っている
if (read(genFile) !== generated()) fail("utilities.gen.ts が古い。mise run gen:styles で作り直す");

// 3. 使う変数はすべて tokens.css にあり、tokens.css の変数はどれも使われている
const tokens = withoutComments(read(join(stylesDir, "tokens.css")));
const defined = new Set([...tokens.matchAll(/(--[a-z0-9-]+)\s*:/g)].map((m) => m[1]));
const used = new Set<string>();
for (const file of walk(stylesDir)) {
  for (const m of withoutComments(read(file)).matchAll(/var\((--[a-z0-9-]+)/g)) {
    const name = m[1] ?? "";
    used.add(name);
    if (!defined.has(name)) fail(`${relative(stylesDir, file)}: ${name} は tokens.css にない`);
  }
}
for (const name of defined) if (!used.has(name)) fail(`tokens.css: ${name} はどこからも使われていない`);

// 4. 画面の側: class は cx(…) か部品の見た目の関数(xxxClass(…))でだけ付け、style 属性は使わない。
//    utilities のクラスはどれも使われている
const sources = [join(frontend, "apps"), join(frontend, "packages")]
  .flatMap(walk)
  .filter((f) => f.includes("/src/") && /\.tsx?$/.test(f) && !/\.(test|gen)\.tsx?$/.test(f) && !f.includes("/gen/"));
const code = new Map(sources.map((f) => [f, read(f)]));
for (const [file, text] of code) {
  const rel = relative(frontend, file);
  const allowed = String.raw`(?:cx|[a-z][A-Za-z]*Class)\(`;
  const pattern = new RegExp(String.raw`className(?:=(?!\{${allowed})|:\s*(?!\s|${allowed}))\S{0,30}`, "g");
  for (const m of text.matchAll(pattern)) {
    fail(`${rel}: className は cx(…) か xxxClass(…) で付ける(${m[0]})`);
  }
  if (/\sstyle=\{/.test(text)) fail(`${rel}: style 属性は使わない(見た目は utilities のクラスで)`);
}
const allCode = [...code.values()].join("\n");
for (const name of utilityNames()) {
  if (!allCode.includes(`"${name}"`)) fail(`utilities.css: .${name} はどこからも使われていない`);
}

if (errors.length > 0) {
  for (const message of errors) console.error(message);
  process.exit(1);
}
