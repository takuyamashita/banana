import "@testing-library/jest-dom/vitest";
import { conflictingClasses } from "@platform/ui/testing";
import { cleanup } from "@testing-library/react";

// どのテストでも、描いた画面の終わりの状態で、同じプロパティを取り合うクラスがないことを確かめてから片付ける
// (RTL の自動の片付けは vite.config.ts で止め、確かめた後にここで片付ける)
afterEach(() => {
  const conflicts = [...document.body.querySelectorAll("[class]")].flatMap((element) =>
    conflictingClasses([...element.classList]).map((conflict) => `<${element.tagName.toLowerCase()}> ${conflict}`),
  );
  cleanup();
  if (conflicts.length > 0) throw new Error(`同じプロパティを取り合うクラスがある:\n${conflicts.join("\n")}`);
});
