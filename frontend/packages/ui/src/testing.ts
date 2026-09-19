import { utilities, type Utility } from "./utilities.gen";

const isUtility = (name: string): name is Utility => Object.hasOwn(utilities, name);

/// 1つの要素に、同じプロパティを書き換えるクラスが2つ付いていないか。付いていれば、どちらが勝つかが
/// utilities.css の並び順で決まってしまう(例: ルーターのリンクが、選んでいるときのクラスを足す)。
/// 付いているものを「a と b が padding-top を取り合う」の形で返す
export function conflictingClasses(classNames: readonly string[]): string[] {
  const owner = new Map<string, string>();
  const conflicts: string[] = [];
  for (const name of classNames) {
    if (!isUtility(name)) continue;
    for (const property of utilities[name]) {
      const other = owner.get(property);
      if (other !== undefined && other !== name) conflicts.push(`${other} と ${name} が ${property} を取り合う`);
      owner.set(property, name);
    }
  }
  return conflicts;
}
