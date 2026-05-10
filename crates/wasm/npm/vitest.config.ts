export default {
  test: {
    environment: "node",
    globals: true,
    include: ["tests/*.test.ts"],
    pool: "threads",
    testTimeout: 30_000
  }
};
