
class Term {
  constructor(type, data = null) {
      this.type = type;
      this.data = data;
  }

  toString() {
      switch (this.type) {
          case 'I': return 'i';
          case 'S': return 's';
          case 'K': return 'k';
          case 'V': return 'v';
          case 'D': return 'd';
          case 'C': return 'c';
          case 'R': return 'r';
          case 'Put':
              return this.data === '\n' ? 'r' : `.${this.data}`;
          case 'App':
              return `\`${this.data[0]}${this.data[1]}`;
          default:
              throw new Error(`Unknown term type: ${this.type}`);
      }
  }
}

// Parser
function parseTerm(s) {
  // remove comments, spaces and newlines, except after "." characters
  s = s.replace(/#.*$/gm, '').replace(/([^.])\s+/g, '$1').replace(/^\s+/, '');
  let pos = 0;

  function parseTermHelper() {
      if (pos >= s.length) {
          throw new Error("Unexpected end of input");
      }

      switch (s[pos]) {
          case 'i': pos++; return new Term('I');
          case 's': pos++; return new Term('S');
          case 'k': pos++; return new Term('K');
          case 'v': pos++; return new Term('V');
          case 'd': pos++; return new Term('D');
          case 'c': pos++; return new Term('C');
          case 'r': pos++; return new Term('R');
          case '.':
              pos++;
              if (pos >= s.length) {
                  throw new Error("Unexpected end of input after '.'");
              }
              const char = s[pos];
              pos++;
              return new Term('Put', char);
          case '`':
              pos++;
              const t0 = parseTermHelper();
              const t1 = parseTermHelper();
              return new Term('App', [t0, t1]);
          default:
              throw new Error(`Unexpected character '${s[pos]}' at position ${pos}`);
      }
  }

  const result = parseTermHelper();
  if (pos < s.length) {
      throw new Error(`Unexpected character at position ${pos}`);
  }
  return result;
}

export { Term, parseTerm };