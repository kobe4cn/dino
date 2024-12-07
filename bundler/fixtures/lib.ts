async function execute(name: string): Promise<string> {
  console.log("Executing lib");
  return `Hello ${name}!`;
}
function not_user() {
  console.log("not user");
}
export { execute };
