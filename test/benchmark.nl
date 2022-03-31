"std.nl" include

fib is
  dup
  1 > lambda
    dup 1 - @fib! swap 2 - @fib! +
  in jump?
in

5 @fib! @std::println!
