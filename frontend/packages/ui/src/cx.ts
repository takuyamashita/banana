import type { Utility } from "./utilities.gen";

/// 見た目は utilities のクラスを並べて付ける。名前は utilities.css から生成した型で確かめる
/// (無いクラスを書くと型エラー)。付けないものは false・undefined で渡す
export function cx(...classes: (Utility | false | null | undefined)[]): string {
  return classes.filter((c): c is Utility => typeof c === "string").join(" ");
}
