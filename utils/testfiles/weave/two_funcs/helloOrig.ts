function hello(name): string {
  function inner(greeting: string): string {
    let myStuff = greeting + ", ";
    return myStuff;
  }
  function inner2(greeting) {
    let myStuff = greeting + ",,, ";
    return myStuff;
  }
  return inner("Hello") + inner2(", HELLOOO") + name;
}
