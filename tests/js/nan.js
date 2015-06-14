var a = 100;
var s = "hey";

function isNaN(n) {
  return n !== n;
}

assert(isNaN(+s));
assert(isNaN(s * 10));
assert(isNaN(s - a));
