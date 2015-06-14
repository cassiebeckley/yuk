var key = "hello";
var object = {
  bleh: "BLEH!",
  andyet: {
    key: key,
    other: key
  }
};

assert_eq(object.bleh, "BLEH!");
assert_eq(object.andyet.key, "hello");
assert_eq(object.andyet.key, object["andyet"]["other"]);
assert_eq(object["andyet"].key, object.andyet["other"]);

object.hello = "Hey!";

assert_eq(object.hello, "Hey!");
assert_eq(object["hello"], object.hello);
assert_eq(object[key], "Hey!");

object[0] = 10;

assert_eq(object[0], 10);
assert_eq(object["0"], 10);
