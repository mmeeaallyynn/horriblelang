"std.nl" include

a is 1 in
b is 5 in

@a! @b! + @std::println!

something is sub
	1 2 3
end in

STACK

sub
	STACK
	"hello from sub!\n" print

	c is 10 in

	100 @a put

	@b! @c! +
	STACK
end

@a! @std::println!

@std::println!

"end!" @std::println!

@something! @something! @something!
STACK