import { parseTerm } from "./term.js";
import { newState, State } from "./am.js";

window.parseTerm = parseTerm;
window.newState = newState;
window.State = State;

function randint(max) {
  return Math.floor(Math.random() * max);
}

window.randomTerm = function() {
  const size = 19;
  let depth = 1;
  let length = 0;
  const acc = [];
  const index = "iksrdcv";
  while(depth) {
    const choice = randint(size);
    if(choice <= 7) {
      if(choice == 7) acc.push(`.${String.fromCharCode(" ".charCodeAt(0) + randint(95))}`);
      else acc.push(index[choice]);
      depth--;
      length++;
    }
    else if(choice == 8) acc.push("`k"), length+=2;
    else if(choice == 9) acc.push("``s"), depth++, length+=3;
    else if(choice == 10) acc.push("``s`kk`k"), length += 8;
    else if(choice == 11) acc.push("``s``s`ks"), depth++, length += 9;
    else if(choice == 12) acc.push("``s``s`ks``s`kk`kk``s`kk`k"), length += 26;
    else if(choice == 13) acc.push("``s``s`ks``s``s`ks``s`kk`ks"), depth++, length += 27;
    else {
      acc.push("`");
      depth++; length++;
    }
  }
  return acc.join("");
}