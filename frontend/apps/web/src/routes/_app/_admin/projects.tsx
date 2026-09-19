import { ProjectService } from "@platform/api-client";
import { createFileRoute } from "@tanstack/react-router";

import { ProjectPage } from "../../../features/project/ProjectPanel";
import { ensure } from "../../../lib/loaders";

export const Route = createFileRoute("/_app/_admin/projects")({
  staticData: { system: "payroll" },
  loader: ({ context, location }) => ensure(context, ProjectService.method.listProjects, {}, location.href),
  component: ProjectPage,
});
