// TODO: use real object literal
var object = function(){};
var key = "hello";

object.hello = "Hey!";

assert_eq(object.hello, "Hey!");
assert_eq(object["hello"], object.hello);
assert_eq(object[key], "Hey!");

object[0] = 10;

assert_eq(object[0], 10);
assert_eq(object["0"], 10);
