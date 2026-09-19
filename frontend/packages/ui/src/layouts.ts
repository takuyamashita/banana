import { cx } from "./cx";

/// 何か所にも出てくる並べ方。CSS のクラスを増やさず、utilities の組み合わせに名前を付けて使い回す

/// 画面の本文の幅と余白
export const pageClass = () => cx("max-w-page", "m-x-auto", "p-5");

/// 入力欄を横に並べ、狭ければ折り返す
export const rowClass = () => cx("grid", "grid-fit", "gap-3", "items-end");

/// ボタンやリンクを横に並べ、狭ければ折り返す
export const clusterClass = () => cx("flex", "wrap", "gap-3", "items-end");

/// 表の数の列(右寄せ・桁をそろえる)
export const numberClass = () => cx("text-right", "tabular-nums");
