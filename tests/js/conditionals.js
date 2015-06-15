var yes = false;

var res = 0;

if (yes) {
  res = 20;
} else {
  res = 30;
}

assert_eq(res, 30);

if (true)
  res = 16;

assert_eq(res, 16);

if (false)
  var hey = "twenty";
else
  var hey = "thirty";

assert_eq(hey, "thirty");

if ("n") {
  res = 1337;
}

assert_eq(res, 1337);

if ("") {} else {
  res = 256;
}

assert_eq(res, 256);

var meh;

if (32) meh = 726; else meh = 35; assert_eq(meh, 726);
if (0) meh = 726; else meh = 35; assert_eq(meh, 35);
