function Person(name, age) {
  this.name = name;
  this.age = age;
}

Person.prototype.toString = function() {
  return "Name: " + this.name + "; Age: " + this.age;
}

var luke = new Person("Luke", 24);

assert_eq(luke + "", "Name: Luke; Age: 24");
