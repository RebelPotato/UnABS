// A sharing abstract machine for unlambda

class Value {
  constructor(type, data = null) {
    this.type = type;
    this.data = data;
  }

  toString() {
    switch (this.type) {
      case "I0":
        return "i";
      case "S0":
        return "s";
      case "K0":
        return "k";
      case "V0":
        return "v";
      case "D0":
        return "d";
      case "C0":
        return "c";
      case "Put0":
        return this.data === "\n" ? "r" : `.${this.data}`;
      case "S1":
        return `\`s${this.data}`;
      case "S2":
        return `\`\`s${this.data[0]}${this.data[1]}`;
      case "K1":
        return `\`k${this.data}`;
      case "D1T":
        return `\`d[${this.data}]`;
      case "D1V":
        return `\`d${this.data}`;
      case "C1":
        return `\`c(${this.data || ""})`;
    }
  }
}

class Kont {
  constructor(type, data, next = null) {
    this.type = type;
    this.data = data;
    this.next = next;
  }

  toString() {
    const acc = [];
    let current = this;
    while (current) {
      acc.push(current);
      current = current.next;
    }
    let wrapped = "()";
    for(const k of acc) {
      switch (k.type) {
        case "BindT":
          wrapped = `\`${wrapped}[${k.data}]`;
          break;
        case "BindV":
          wrapped = `\`${k.data}${wrapped}`;
          break;
        case "BindW":
          wrapped = `\`${wrapped}${k.data}`;
          break;
        case "SWait":
          wrapped = `\`${wrapped}\`${k.data[0]}${k.data[1]}`;
          break;
      }
    }
    return wrapped;
  }
}

class State {
  constructor(type, data, kont, stdout) {
    this.type = type;
    this.data = data;
    this.kont = kont;
    this.stdout = stdout;
  }

  toString() {
    let result = `State: ${this.type}\n`;
    switch (this.type) {
      case "Eval":
        result += `Term: [${this.data}]\n`;
        break;
      case "ApplyT":
        result += `Value: ${this.data[0]}\nTerm: [${this.data[1]}]\n`;
        break;
      case "ApplyV":
        result += `Value: ${this.data[0]}\nWalue: ${this.data[1]}\n`;
        break;
      case "ApplyK":
        result += `Value: ${this.data}\n`;
        break;
    }
    result += `Kont: ${this.kont || "()"}`;
    return result;
  }

  step() {
    switch (this.type) {
      case "Eval":
        return evaluate(this.data, this.kont, this.stdout);
      case "ApplyT":
        return applyT(this.data[0], this.data[1], this.kont, this.stdout);
      case "ApplyV":
        return applyV(this.data[0], this.data[1], this.kont, this.stdout);
      case "ApplyK":
        if (this.kont) {
          return applyK(this.kont, this.data, this.stdout);
        } else {
          return this.data;
        }
    }
  }

  run() {
    let state = this;
    while (true) {
      const result = state.step();
      if (result instanceof Value) {
        return result;
      }
      state = result;
    }
  }
}

function evaluate(term, kont, stdout) {
  switch (term.type) {
    case "I":
    case "S":
    case "K":
    case "V":
    case "D":
    case "C":
      return new State("ApplyK", new Value(`${term.type}0`), kont, stdout);
    case "R":
      return new State("ApplyK", new Value("Put0", "\n"), kont, stdout);
    case "Put":
      return new State("ApplyK", new Value("Put0", term.data), kont, stdout);
    case "App":
      return new State(
        "Eval",
        term.data[0],
        new Kont("BindT", term.data[1], kont), stdout
      );
    default:
      throw new Error(`Unknown term type: ${term.type}`);
  }
}

function applyT(v, t, k, stdout) {
  if (v.type === "D0") {
    return new State("ApplyK", new Value("D1T", t), k, stdout);
  } else {
    return new State("Eval", t, new Kont("BindV", v, k), stdout);
  }
}

function applyV(v, w, k, stdout) {
  switch (v.type) {
    case "I0":
      return new State("ApplyK", w, k, stdout);
    case "Put0":
      stdout.write(v.data);
      return new State("ApplyK", w, k, stdout);
    case "K0":
      return new State("ApplyK", new Value("K1", w), k, stdout);
    case "K1":
      return new State("ApplyK", v.data, k, stdout);
    case "V0":
      return new State("ApplyK", new Value("V0"), k, stdout);
    case "C0":
      return new State("ApplyV", [w, new Value("C1", k)], k, stdout);
    case "C1":
      return new State("ApplyK", w, v.data, stdout);
    case "D0":
      return new State("ApplyK", new Value("D1V", w), k, stdout);
    case "D1T":
      return new State("Eval", v.data, new Kont("BindW", w, k), stdout);
    case "D1V":
      return new State("ApplyV", [v.data, w], k, stdout);
    case "S0":
      return new State("ApplyK", new Value("S1", w), k, stdout);
    case "S1":
      return new State("ApplyK", new Value("S2", [v.data, w]), k, stdout);
    case "S2":
      return new State(
        "ApplyV",
        [v.data[0], w],
        new Kont("SWait", [v.data[1], w], k),
        stdout
      );
  }
}

function applyK(k, w, stdout) {
  switch (k.type) {
    case "BindT":
      return new State("ApplyT", [w, k.data], k.next, stdout);
    case "BindV":
      return new State("ApplyV", [k.data, w], k.next, stdout);
    case "BindW":
      return new State("ApplyV", [w, k.data], k.next, stdout);
    case "SWait":
      return new State("ApplyV", k.data, new Kont("BindV", w, k.next), stdout);
  }
}

function newState(term, stdout) {
  return new State("Eval", term, null, stdout);
}

// Export the main function and other necessary components
export { newState, Value, Kont, State };
