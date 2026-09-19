import { ProjectService, type Project } from "@platform/api-client";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { renderWithApi } from "../../test-utils";
import { ProjectPanel } from "./ProjectPanel";

test("登録すると一覧に加わる", async () => {
  const projects: Omit<Project, "$typeName">[] = [{ projectId: 1n, name: "案件A" }];
  renderWithApi(<ProjectPanel />, ({ service }) => {
    service(ProjectService, {
      listProjects: () => ({ projects }),
      createProject: ({ name }) => {
        projects.push({ projectId: 2n, name });
        return { projectId: 2n };
      },
    });
  });
  const user = userEvent.setup();
  await screen.findByText("#1 案件A");

  await user.type(screen.getByLabelText("案件名"), "新宿の倉庫");
  await user.click(screen.getByRole("button", { name: "登録する" }));

  expect(await screen.findByText("#2 新宿の倉庫")).toBeInTheDocument();
  expect(screen.getByLabelText("案件名")).toHaveValue("");
});
