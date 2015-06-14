var a = 100;
var b = 235.6;
var s = "hey";

assert_eq(a + 10 + s, "110hey");
assert_eq(a + s + 10, "100hey10");
assert_eq("20" + 15, "2015");

//assert_eq(true.toString(), "true");
//assert_eq(true + "", "true");

assert_eq(s.child, undefined);

assert_eq(s.child = 10, undefined);
assert_eq(s.child, undefined);
