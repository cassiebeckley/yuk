function ten() {
  return 10
}

assert_eq(ten(), 10);

function scope() {
  var obj = {twenty: 20};
  return obj;
}

assert_eq(scope().twenty, 20);

function yes(a) {
  hello = a
}

assert_eq(yes(25), undefined);
assert_eq(hello, 25);

function add(a, b) {
  return a + b;
}

assert_eq(add(7, 4), 11);

function counter() {
  var count = 0;
  return function() {
    // TODO: ++
    var old_count = count;
    count = count + 1;
    return old_count;
  };
}

var count = counter();

assert_eq(count(), 0);
assert_eq(count(), 1);
assert_eq(count(), 2);
