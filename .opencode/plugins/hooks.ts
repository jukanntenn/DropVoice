// OpenCode plugin — thin adapter over prek (format on write/edit).
//
// The formatter definitions live exclusively in prek.toml (repo root +
// apps/*/packages/*/prek.toml). OpenCode's event model has no blocking Stop
// channel, so lint enforcement falls back to prek's pre-commit gate and CI.

export const HooksPlugin = async ({ $ }: { $: any }) => {
  return {
    "tool.execute.after": async (input: any, output: any) => {
      if (input.tool !== "write" && input.tool !== "edit") return
      const filePath: string | undefined =
        output.args?.filePath ?? output.args?.file_path
      if (!filePath) return
      // Best-effort: format via prek, never block the session.
      await $`prek run --group format --files ${filePath}`.quiet().nothrow()
    },
  }
}
