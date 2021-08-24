def fact(a)
  puts(a)
  if a == 1
    1
  else
    a * fact(a-1)
  end
end

assert(fact(5), 120)

def self1
  puts(self)
end

self1()

class Foo
  puts(self)
  class Bar
    puts(self)
  end
end

assert(self1(), nil)

a = 1
def foo
  a
end
assert(foo(), 1)

a = 1
def foo
  a
  a = 2
end
foo()
assert(a, 1)

class Foo
end
Foo.new
Foo.new
puts(Foo.new)

a = 1
class Foo
  a = 2
  def bar(b)
    b*2
  end

  def bar2
    a
  end
end

assert(Foo.new.bar(5), 10)
assert(Foo.new.bar2, 2)

assert(1, 1)

a = 1
class Foo
  a = 2
  def bar(b)
    b*2
  end

  def get_a
    a
  end
end

assert(a, 1)
assert(Foo.new.bar(5), 10)
assert(Foo.new.get_a(), 2)

a = 6;
b = 2;c = 1;
assert(a/b-c, 2)

assert('34'.to_i, 34)

puts(34.to_s)

class Bar
end
puts(Bar.class)

a =
  3.times do
  puts('hello')
end
assert(a, nil)

a =
  3.times do |n|
  puts(n)
end
assert(a, nil)

a = 0
255.times do |n|
  a = a + n
end
assert(a, 32385)

a = 0
b = 0
24.times do |n|
  b = b + n + a
  a = b
end

assert(a, 16777191)

assert([1, 'string', 3, 4].len, 4)

puts([1, 'string', 3, 4][1])

v = ['one', 2, 'three', 4]
v.each do |c|
  puts(c)
end

class Vec
  @xxx=100
  def set_xxx(x)
    @xxx = x
  end
  def len(x,y)
    def sq(x)
      x*x
    end
    sq(x)+sq(y)
  end
  def get_xxx
    @xxx
  end
end
foo1 = Vec.new
foo1.set_xxx(1)
assert(25, foo1.len(3,4))
foo1.set_xxx(777)
foo2 = Vec.new
assert(777, foo1.get_xxx)
foo2.set_xxx(999)
assert(777, foo1.get_xxx)
assert(999, foo2.get_xxx)

class Car
  def setName(str)
    @name = str
  end

  def getName
    @name
  end
end

car1 = Car.new
car1.setName('Legacy')

car2 = Car.new
car2.setName('XV')
assert(car2.getName, 'XV')
assert(car1.getName, 'Legacy')

class Car
  def setName(str)
    @name = str
  end

  def getName
    @name
  end
end

puts(car1 = Car.new)
puts(car1.setName('Legacy'))
puts(car1.instance_variables)

class Car
  @@class_var = 2

  def set_class_var(i)
    @@class_var = i
  end

  def get_class_var
    @@class_var
  end
end

car1 = Car.new
car1.set_class_var(22222)
assert(car1.get_class_var, 22222)

class A
  @xxx=100
  def set_xxx(x)
    @xxx = x
  end
  def len(x,y)
    def sq(x)
      x*x
    end
    sq(x)+sq(y)
  end
  def get_xxx
    @xxx
  end
end

class B < A
end
foo1 = A.new
foo1.set_xxx(1)
assert(25, foo1.len(3,4))
foo1.set_xxx(777)
foo2 = B.new
# assert(777, foo1.get_xxx)
# foo2.set_xxx(999)
# assert(777, foo1.get_xxx)
# assert(999, foo2.get_xxx)
