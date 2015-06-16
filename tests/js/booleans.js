var yes = true;

assert_eq(yes, true);
assert_eq(yes && true, true);

assert_eq(true && true, true);
assert_eq(false && true, false);

assert_eq(true && 10, 10);

assert_eq(1 + 1 && true, true);
assert_eq(true && 1 + 1, 2);

assert_eq(true || true, true);
assert_eq(true || false, true);

assert_eq(true || 15, true);
assert_eq(false || 15, 15);

// assert_eq(!true, false);

// var test = 0;

// false && test = 2;
// assert_eq(test, 0);

// true && test = 2;
// assert_eq(test, 2);

// true || test = 15;
// assert_eq(test, 2);

// false || test = 15;
// assert_eq(test, 15);
