# 0007 CSS はカスケードレイヤーと utilities だけで書き、決まりを lint で守る

- 日付: 2026-09-20
- 状態: 採用

## 決定

- CSS は `packages/ui/src/styles/` にだけ置き、入口の `index.css` で層を付けて読み込む。層は `@layer vendor, reset, tokens, base, utilities;` の順(後ろほど強い)。アプリは `main.tsx` で `@platform/ui/styles.css` を1回だけ読み込む。
- 部品ごとの CSS のクラス(`.ui-card` など)は作らない。見た目は utilities のクラス(`p-t-1` = padding-top: var(--space-1))を並べて書き、使い回す単位は React の部品(`Button`・`Card`)と、組み合わせに名前を付けた関数(`buttonClass()`・`rowClass()`)にする。
- 見た目の値は tokens にだけ書く。base と utilities は `var(--…)` で読み、長さの単位や色を直に書かない。状態(`:disabled` など)の見た目は base で要素ごとに1つに決め、状態つきのクラスは作らない。
- クラスは `cx("p-t-1", …)` で付ける。引数の型は `utilities.css` から生成する(`mise run gen:styles`)ので、無いクラスは型エラーになる。
- 決まりは次の3つで守る。
  - stylelint: 層(ファイル)ごとに書いてよいものを絞る(tokens は `:root` の変数だけ、base はクラスなし、utilities は1クラス1ルールで子孫・状態なし、どこでも `!important` なし)。
  - `packages/ui/scripts/styles.ts`: 層の順と読み込み、styles/ の外の CSS、変数の定義と未使用、生成した型の鮮度、`className` を `cx` か `xxxClass` でだけ付けること、`style` 属性、使われていない utilities。
  - Vitest: どのテストでも、描いた画面に同じプロパティを取り合うクラス(`bg-transparent` と `bg-accent` など)が1つの要素に付いていないことを確かめる(`test-setup.ts`)。
- フロントエンドの依存の向きは dependency-cruiser(`.dependency-cruiser.cjs`)で確かめる。features 同士・lib から上・layout から features・packages から apps・e2e から画面のコード・main.tsx 以外からの CSS の読み込み・循環・どこからも使われないファイルを禁止する。oxlint の `no-restricted-imports` の上書きはやめた。

## 理由

- 層で勝ち負けを決めれば、詳細度の競争や `!important` がいらない。層に入らない CSS はどの層にも勝つので、CSS の置き場所と読み込み口を1つにする。
- 部品ごとのクラスは、部品の数だけ増えて使われなくなっても残る。使い回しを React の部品に任せれば、CSS は値(tokens)と小さなクラスだけで済み、使われていないクラスは機械的に見つけられる。
- 同じ要素に同じプロパティのクラスが2つ付くと、どちらが勝つかが `utilities.css` の並び順で決まる。実際に、ルーターのリンクが選んでいるときのクラスを足す(置き換えない)ため、選んでいるメニューの文字が背景と同じ色になっていた(移し替えの前は、選んでいるメニューが強調されていなかった)。スクリーンショットの比較で見つけ、テストで常に確かめるようにした。
- 依存の向きは、features が増えるたびに oxlint の上書きを書き足す形では漏れる。dependency-cruiser なら「自分以外の features」を1つの決まりで書け、循環も見つかる(実際に、ルートに渡す context の型を router.tsx に置いていたため、lib・routes・router の間で循環していた。型を lib/ に移した)。

## 捨てた案

- Tailwind: 使ったクラスだけを作り、状態つきのクラスも使えるが、層は Tailwind の決まりに合わせることになり、道具も増える。今の画面で要るクラスは39個で、手で書ける。
- 部品ごとのクラスを components 層に置く: 上の理由で、使い回しは React の部品に任せる。

## 補足

- dependency-cruiser は TypeScript 7 の API をまだ使えないので、swc(`@swc/core`)で TS を読む。「TypeScript が見つからない」という警告が出るが、すべてのファイルを読めていることは確かめた。
