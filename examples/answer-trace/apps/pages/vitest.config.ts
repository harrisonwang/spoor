import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    include: ["functions/_lib/__tests__/**/*.test.ts"],
  },
});
