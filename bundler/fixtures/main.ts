import { execute } from "./lib.ts";
async function main() {
  console.log("Executing main");
  const result = await execute("world");
  console.log(result);
}

export { main };
